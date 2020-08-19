use crate::backend::{Backend, HasSqlType};
use crate::error::AnyResult;
use crate::types::{NotNull, Nullable};
use crate::values::RawValue;

/// Deserialize a single field of a given SQL type.
pub trait FromSql<'r, SqlTy, Db: Backend>: Sized
where
    Db: HasSqlType<SqlTy>,
{
    fn from_sql(metadata: &Db::TypeMetadata, value: Db::RawValue<'r>) -> AnyResult<Self>;
}

impl<'r, RustTy, SqlTy, Db> FromSql<'r, Nullable<SqlTy>, Db> for Option<RustTy>
where
    Db: Backend + HasSqlType<SqlTy>,
    RustTy: FromSql<'r, SqlTy, Db>,
    SqlTy: NotNull,
{
    fn from_sql(metadata: &Db::TypeMetadata, value: Db::RawValue<'r>) -> AnyResult<Self> {
        if value.is_null() {
            Ok(None)
        } else {
            RustTy::from_sql(metadata, value).map(Some)
        }
    }
}
