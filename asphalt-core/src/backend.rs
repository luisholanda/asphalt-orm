use crate::query::{BindCollector, PreparableQuery, QueryWriter};
use crate::types::{NotNull, Nullable};
use crate::values::RawValue;
use futures_util::future::BoxFuture;

/// A database backend.
///
/// This trait represents a specific backend protocol (e.g. MySQL and Postgres).
pub trait Backend: Sized + TypeMetadata {
    /// The type of query used to communicate with the backend.
    ///
    /// For most backends this will be `String`, but is made generic so that
    /// more structured formats can be supported if necessary, e.g. in a Cassandra
    /// backend, this could also hold the desired consistency level.
    type Query: PreparableQuery<Self>;
    /// Type used to write SQL queries for this backend.
    type QueryWriter: QueryWriter<Self>;
    /// Type used as placeholder for bind parameters in an SQL query.
    type BindName;
    /// Type used as bind parameters collector.
    type BindCollector: BindCollector<Self>;
    /// The type of raw values used to communicate with the backend.
    ///
    /// See [`RawValue`] for more info.
    type RawValue: RawValue<Self>;
    /// Data contained in a row.
    type RowData;
}

/// Indicates that a sql type exists in the database.
pub trait HasSqlType<Ty>: TypeMetadata {
    fn metadata(lookup: &Self::MetadataLookup) -> BoxFuture<'_, Self::TypeMetadata>;
}

impl<SqlTy, Db> HasSqlType<Nullable<SqlTy>> for Db
where
    Db: HasSqlType<SqlTy>,
    SqlTy: NotNull,
{
    fn metadata(lookup: &Self::MetadataLookup) -> BoxFuture<'_, Self::TypeMetadata> {
        <Db as HasSqlType<SqlTy>>::metadata(lookup)
    }
}

/// How a [`Backend`] stores type metadata?
pub trait TypeMetadata {
    /// Metadata information about a database type.
    ///
    /// This in postgres would be the OID of the type.
    type TypeMetadata: Clone;
    /// The type used for runtime lookup of metadata.
    type MetadataLookup;
}

/// Opaque type that holds a bind name.
pub struct BindName<Db: Backend> {
    inner: Db::BindName,
}

impl<Db: Backend> BindName<Db> {
    pub(crate) fn new(name: Db::BindName) -> Self {
        Self { inner: name }
    }

    pub(crate) fn inner(&self) -> &Db::BindName {
        &self.inner
    }
}

impl<Db: Backend> Copy for BindName<Db> where Db::BindName: Copy + Clone {}

impl<Db: Backend> Clone for BindName<Db>
where
    Db::BindName: Clone,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
    }
}
