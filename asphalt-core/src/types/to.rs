use crate::backend::{Backend, HasSqlType};
use crate::error::AnyResult;
use crate::types::{NotNull, Nullable};
use crate::values::RawValue;

/// Serializes a single value to be sent to the database.
pub trait ToSql<SqlTy, Db: Backend>
where
    Db: HasSqlType<SqlTy>,
{
    /// Serialize this value in the format expected by the backend.
    fn to_sql<'a>(
        &'a self,
        metadata: &Db::TypeMetadata,
        collector: &'a mut Db::BindCollector,
    ) -> AnyResult<Db::RawValue<'a>>;
}

/// Any `T` which implements `ToSql<ST>` also implements `ToSql<Nullable<ST>>`.
impl<RustTy, SqlTy, Db> ToSql<Nullable<SqlTy>, Db> for RustTy
where
    SqlTy: NotNull,
    RustTy: ToSql<SqlTy, Db> + NotNull,
    Db: Backend + HasSqlType<SqlTy>,
{
    fn to_sql<'a>(
        &'a self,
        metadata: &Db::TypeMetadata,
        collector: &'a mut Db::BindCollector,
    ) -> AnyResult<Db::RawValue<'a>> {
        self.to_sql(metadata, collector)
    }
}

/// `Option<T>` implements `ToSql<Nullable<ST>>` if `T` implements `ToSql<ST>`.
impl<RustTy, SqlTy, Db> ToSql<Nullable<SqlTy>, Db> for Option<RustTy>
where
    SqlTy: NotNull,
    RustTy: ToSql<SqlTy, Db>,
    Db: Backend + HasSqlType<SqlTy>,
{
    fn to_sql<'a>(
        &'a self,
        metadata: &Db::TypeMetadata,
        collector: &'a mut Db::BindCollector,
    ) -> AnyResult<Db::RawValue<'a>> {
        if let Some(value) = self {
            value.to_sql(metadata, collector)
        } else {
            Ok(<Db::RawValue<'a>>::null_value())
        }
    }
}
