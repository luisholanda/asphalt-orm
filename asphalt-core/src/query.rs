use crate::backend::{Backend, BindName, HasSqlType};
use crate::connection::RawConnection;
use crate::error::QueryResult;
use crate::types::ToSql;
use crate::utils::CowMut;
use futures_util::future::BoxFuture;
use std::cell::Cell;
use std::marker::PhantomData;

/// A constructed query.
pub struct Query<Db: Backend> {
    /// The final constructed query.
    pub inner: Db::Query,
    /// Bind parameters that should be sent with the query.
    pub binds: Db::BindCollector,
}

/// A database query for a backend.
pub trait PreparableQuery<Db: Backend>: Sized {
    /// The type of prepared queries.
    type Prepared: Clone;

    /// Prepare the query, binding it to the given connection.
    fn prepare<Conn>(self, conn: &Conn) -> BoxFuture<QueryResult<Self::Prepared>>
    where
        Conn: RawConnection<Backend = Db>;

    fn from_prepared(prepared: Self::Prepared, binds: Db::BindCollector) -> Self;
}

/// Type alias for a prepared query.
pub type PreparedQuery<Db> = <<Db as Backend>::Query as PreparableQuery<Db>>::Prepared;

/// Manages serialization of bind parameters during query construction.
pub trait BindCollector<Db: Backend>: Default {
    /// Add a new bind parameter to the collector.
    fn push_bound_value<SqlTy, RustTy>(
        &mut self,
        bind: &RustTy,
        metadata_lookup: &Db::MetadataLookup,
    ) -> BoxFuture<'_, QueryResult<Db::BindName>>
    where
        Db: HasSqlType<SqlTy>,
        RustTy: ToSql<SqlTy, Db>;
}

/// Constructs a SQL query from its parts.
pub trait QueryWriter<Db: Backend>: Default {
    /// Adds `sql` to the end of the query being constructed.
    fn push_sql(&mut self, sql: &str);
    /// Quote `identifier` and add it to the end of the query being constructed.
    fn push_identifier(&mut self, identifier: &str);
    /// Add a placeholder `name` for a bind parameter to the end of the query being constructed.
    fn push_bind_param(&mut self, name: &Db::BindName);
    /// Returns the constructed query.
    fn finish(self) -> Db::Query;
}

/// A builder of SQL queries.
pub struct QueryBuilder<'q, 'b, Db: Backend> {
    metadata_lookup: &'q Db::MetadataLookup,
    writer: CowMut<'b, Db::QueryWriter>,
    collector: CowMut<'b, Db::BindCollector>,
    safe_to_cache: CowMut<'b, bool>,
    // Make QueryBuilder invariant over 'q.
    _marker: PhantomData<Cell<&'q ()>>,
}

impl<'q, Db: Backend> QueryBuilder<'q, 'static, Db> {
    pub(crate) fn new(metadata_lookup: &'q Db::MetadataLookup) -> Self {
        Self {
            metadata_lookup,
            safe_to_cache: CowMut::Owned(true),
            collector: CowMut::Owned(Default::default()),
            writer: CowMut::Owned(Default::default()),
            _marker: PhantomData,
        }
    }

    /// Returns whether the query constructed by this builder is safe to cache.
    pub fn is_safe_to_cache(&self) -> bool {
        *self.safe_to_cache
    }

    /// Finish the construction of the query.
    pub fn finish(self) -> Query<Db> {
        match (self.writer, self.collector) {
            (CowMut::Borrowed(_), _) | (_, CowMut::Borrowed(_)) => {
                unreachable!("Constructed a QueryBuilder with a &'static mut.")
            }
            (CowMut::Owned(writer), CowMut::Owned(collector)) => Query {
                inner: writer.finish(),
                binds: collector,
            },
        }
    }
}

impl<'q, 'b, Db: Backend> QueryBuilder<'q, 'b, Db> {
    /// Creates a new borrowed instance of [`QueryBuilder`].
    ///
    /// Effectively copies `self` with a narrower lifetime.
    pub fn reborrow(&mut self) -> QueryBuilder<'q, '_, Db> {
        QueryBuilder {
            metadata_lookup: self.metadata_lookup,
            writer: self.writer.reborrow(),
            collector: self.collector.reborrow(),
            safe_to_cache: self.safe_to_cache.reborrow(),
            _marker: self._marker,
        }
    }

    /// Mark the current query being constructed as unsafe to store in the prepared statement cache.
    ///
    /// We want to cache prepared statements as much as possible. However, is important to ensure
    /// that this doesn't result in unbounded memory usage on the database server. To ensure this
    /// is the case, ANY logical query which could generate a potentially unbounded number of
    /// prepared statements MUST call this method. Examples of AST nodes which do this are:
    ///
    /// * Literal SQL statements, as we don't have a way to know if the query string is dynamic
    /// or not, so we assume it is.
    /// * Insert statements are unbounded due to the variable number of records being inserted.
    pub fn unsafe_to_cache(&mut self) {
        *self.safe_to_cache = false;
    }

    /// Push the given SQL string to the end of the query being constructed.
    pub fn push_sql(&mut self, sql: &str) {
        self.writer.push_sql(sql)
    }

    /// Push the given identifier to the end of the query being constructed.
    ///
    /// The identifier will be quoted as expected by the backend.
    pub fn push_identifier(&mut self, identifier: &str) {
        self.writer.push_identifier(identifier);
    }

    /// Push a value onto the given query to be send alongside the SQL.
    ///
    /// The returned name can be used with [`QueryBuilder::push_bind_name`] to
    /// re-reference this parameter.
    pub async fn push_bind_param<ST, RT>(&mut self, bind: &RT) -> QueryResult<BindName<Db>>
    where
        Db: HasSqlType<ST>,
        RT: ToSql<ST, Db>,
    {
        let name = self
            .collector
            .push_bound_value::<ST, _>(bind, self.metadata_lookup)
            .await?;

        self.writer.push_bind_param(&name);

        Ok(BindName::new(name))
    }

    /// Push an already bound parameter into the query being constructed.
    pub fn push_bind_name(&mut self, name: &BindName<Db>) {
        self.writer.push_bind_param(name.inner());
    }
}
