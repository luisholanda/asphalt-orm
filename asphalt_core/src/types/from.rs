use crate::backend::{Backend, HasSqlType};
use crate::error::AnyResult;
use crate::types::{NotNull, Nullable};
use crate::values::RawValue;

/// Deserialize a single field of a given SQL type.
pub trait FromSql<SqlTy, Db: Backend>: Sized
where
    Db: HasSqlType<SqlTy>,
{
    fn from_sql(
        metadata: &Db::TypeMetadata,
        extractor: &mut Db::RowData,
        value: Db::RawValue,
    ) -> AnyResult<Self>;
}

impl<RustTy, SqlTy, Db> FromSql<Nullable<SqlTy>, Db> for Option<RustTy>
where
    Db: Backend + HasSqlType<SqlTy>,
    RustTy: FromSql<SqlTy, Db>,
    SqlTy: NotNull,
{
    fn from_sql(
        metadata: &Db::TypeMetadata,
        extractor: &mut Db::RowData,
        value: Db::RawValue,
    ) -> AnyResult<Self> {
        if value.is_null() {
            Ok(None)
        } else {
            RustTy::from_sql(metadata, extractor, value).map(Some)
        }
    }
}
