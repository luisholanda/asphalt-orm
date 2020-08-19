use crate::backend::{Backend, TypeMetadata};
use crate::error::{Error, QueryResult};
use crate::query::{PreparableQuery, PreparedQuery, Query, QueryBuilder};
use futures_util::future::{Future, LocalBoxFuture};

mod row;
mod transaction;

#[doc(inline)]
pub use self::row::{Row, RowStream};
#[doc(inline)]
pub use self::transaction::{
    IsolationLevel, NoopTransactionManager, Transaction, TransactionConfig, TransactionManager,
};

pub type EstablishResult<Conn> = Result<Conn, <Conn as RawConnection>::EstablishError>;

/// A low level connection to a backend.
pub trait RawConnection: Sized + Send + Sync {
    /// The backend of this connection.
    type Backend: Backend;
    /// The transaction manager of this connection.
    type TransactionManager: TransactionManager<Self>;
    /// The type of row returned by the connection.
    type Row: Row;
    /// The configuration necessary to establish a connection.
    ///
    /// In many cases, this can be `str`.
    type Config: Sized;
    type EstablishError: std::error::Error + Send + Sync;

    /// Establish a new connection.
    fn establish(config: Self::Config) -> LocalBoxFuture<'static, EstablishResult<Self>>;

    /// Returns the transaction manager of this connection.
    fn transaction_manager(&self) -> &Self::TransactionManager;

    /// Execute a simple SQL query.
    fn simple_execute<'s>(&'s self, sql: &'s str) -> LocalBoxFuture<'s, QueryResult<()>>;

    /// Execute the given query, returning the number of affected rows.
    fn execute(&self, query: Query<Self::Backend>) -> LocalBoxFuture<'_, QueryResult<u64>>;

    /// Execute the given query, returning the result set.
    fn query(
        &self,
        query: Query<Self::Backend>,
    ) -> LocalBoxFuture<'_, QueryResult<RowStream<'_, Self>>>;

    /// Returns an instance of the type used to lookup type metadata.
    fn metadata_lookup(&self) -> &<Self::Backend as TypeMetadata>::MetadataLookup;
}

/// A mid level connection to a backend.
///
/// This provides the minimum amount of functionality required to make and execute SQL queries
/// without having to wire everything together. Also, provides a panic-safe execution of transactional
/// futures.
///
/// Most users will prefer to use something more high-level than this, using something like the DSL
/// provided by `asphalt_dsl`. Some users, though, will find this interface very useful in cases of
/// high dynamic SQL query where many backends need to be supported.
pub struct Connection<Db>
where
    Db: Backend,
{
    conn: Db::RawConnection,
}

impl<Db> Connection<Db>
where
    Db: Backend,
{
    /// Establish a new connection to the backend.
    pub async fn establish(
        config: <Db::RawConnection as RawConnection>::Config,
    ) -> Result<Self, <Db::RawConnection as RawConnection>::EstablishError> {
        let conn = <Db::RawConnection as RawConnection>::establish(config).await?;

        Ok(Self { conn })
    }

    /// Is this connection in a broken state?
    ///
    /// See [`TransactionManager`] for more info.
    pub fn is_broken(&self) -> bool {
        self.conn.transaction_manager().is_broken()
    }

    /// Create a new [`QueryBuilder`] bound to this connection.
    pub fn query_builder(&self) -> QueryBuilder<'_, 'static, Db> {
        QueryBuilder::new(self.conn.metadata_lookup())
    }

    /// Prepares the query stored inside a [`QueryBuilder`], returning the prepared statement and
    /// the bound parameters.
    pub async fn prepare<'c>(
        &'c self,
        query: QueryBuilder<'c, 'static, Db>,
    ) -> QueryResult<(PreparedQuery<Db>, Db::BindCollector)> {
        let Query { inner, binds } = query.finish();
        let prepared = inner.prepare(&self.conn).await?;

        Ok((prepared, binds))
    }

    /// Executes the query stored inside a [`QueryBuilder`], returning the result set as a stream.
    pub async fn query<'c>(
        &'c self,
        query: QueryBuilder<'c, 'static, Db>,
    ) -> QueryResult<RowStream<'c, Db::RawConnection>> {
        self.conn.query(query.finish()).await
    }

    /// Executes the query stored inside a [`QueryBuilder`], returning the number of affected rows.
    pub async fn executes<'c>(&'c self, query: QueryBuilder<'c, 'static, Db>) -> QueryResult<u64> {
        self.conn.execute(query.finish()).await
    }

    /// Executes the given future inside of a database transaction.
    ///
    /// If there is already an open transaction, a savepoint will be created instead.
    ///
    /// If the transaction fails to commit due to a serialization failure, a rollback
    /// will be attempted as is expected. When the rollback succeeds, the original error
    /// will be returned, otherwise, the rollback error will be returned instead. In
    /// the second case, the connection should be considered broken as it contains an
    /// uncommitted unabortable open transaction.
    ///
    /// # Panics
    ///
    /// If the received future panics, the future returned by this function will try
    /// to rollback the transaction before resuming the panic.
    pub fn transaction<F, T, E>(&self, fut: F) -> Transaction<'_, Db::RawConnection, F>
    where
        F: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Send + From<Error>,
    {
        Transaction::new(&self.conn, fut)
    }
}
