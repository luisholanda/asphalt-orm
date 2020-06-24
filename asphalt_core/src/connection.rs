use crate::backend::{Backend, TypeMetadata};
use crate::error::{Error, QueryResult};
use crate::query::{PreparableQuery, PreparedQuery, Query, QueryBuilder};
use futures_util::future::{BoxFuture, Future};

mod row;
mod transaction;

#[doc(inline)]
pub use self::row::{Row, RowColumn, RowStream};
#[doc(inline)]
pub use self::transaction::{
    IsolationLevel, NoopTransactionManager, Transaction, TransactionConfig, TransactionManager,
};

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
    type Config: ?Sized;

    /// Establish a new connection.
    fn establish(config: &Self::Config) -> BoxFuture<'_, QueryResult<Self>>;

    /// Returns the transaction manager of this connection.
    fn transaction_manager(&self) -> &Self::TransactionManager;

    /// Execute a simple SQL query.
    fn simple_execute(&self, sql: &str) -> BoxFuture<'_, QueryResult<()>>;

    /// Execute the given query, returning the number of affected rows.
    fn execute(&self, query: Query<Self::Backend>) -> BoxFuture<'_, QueryResult<usize>>;

    /// Execute the given query, returning the result set.
    fn query(&self, query: Query<Self::Backend>)
        -> BoxFuture<'_, QueryResult<RowStream<'_, Self>>>;

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
pub struct Connection<Conn>
where
    Conn: RawConnection,
{
    conn: Conn,
}

impl<Conn> Connection<Conn>
where
    Conn: RawConnection,
{
    /// Establish a new connection to the backend.
    pub async fn establish(config: &Conn::Config) -> QueryResult<Self> {
        let conn = Conn::establish(config).await?;

        Ok(Self { conn })
    }

    /// Is this connection in a broken state?
    ///
    /// See [`TransactionManager`] for more info.
    pub fn is_broken(&self) -> bool {
        self.conn.transaction_manager().is_broken()
    }

    /// Create a new [`QueryBuilder`] bound to this connection.
    pub fn query_builder(&self) -> QueryBuilder<'_, 'static, Conn::Backend> {
        QueryBuilder::new(self.conn.metadata_lookup())
    }

    /// Prepares the query stored inside a [`QueryBuilder`], returning the prepared statement and
    /// the bound parameters.
    pub async fn prepare<'c>(
        &'c self,
        query: QueryBuilder<'c, 'static, Conn::Backend>,
    ) -> QueryResult<(
        PreparedQuery<Conn::Backend>,
        <Conn::Backend as Backend>::BindCollector,
    )> {
        let Query { inner, binds } = query.finish();
        let prepared = inner.prepare(&self.conn).await?;

        Ok((prepared, binds))
    }

    /// Executes the query stored inside a [`QueryBuilder`], returning the result set as a stream.
    pub async fn query<'c>(
        &'c self,
        query: QueryBuilder<'c, 'static, Conn::Backend>,
    ) -> QueryResult<RowStream<'c, Conn>> {
        self.conn.query(query.finish()).await
    }

    /// Executes the query stored inside a [`QueryBuilder`], returning the number of affected rows.
    pub async fn executes<'c>(
        &'c self,
        query: QueryBuilder<'c, 'static, Conn::Backend>,
    ) -> QueryResult<usize> {
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
    pub fn transaction<F, T, E>(&self, fut: F) -> Transaction<'_, Conn, F>
    where
        F: Future<Output = Result<T, E>> + Send,
        T: Send,
        E: Send + From<Error>,
    {
        Transaction::new(&self.conn, fut)
    }
}
