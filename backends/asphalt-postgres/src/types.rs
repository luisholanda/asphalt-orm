use crate::Pg;
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::error::{AnyResult, QueryResult};
use asphalt_core::types::*;
use asphalt_core::LocalBoxFuture;
use tokio_postgres::types::{FromSql as PgFromSql, ToSql as PgToSql, Type};

macro_rules! delegate_to_pgtosql {
    ($($($rust_ty: ty),+ => $asp_ty: ty => $pg_ty: ident);+) => {$(
        impl HasSqlType<$asp_ty> for Pg {
            fn metadata(
                _: &Self::MetadataLookup,
            ) -> LocalBoxFuture<'_, QueryResult<Self::TypeMetadata>> {
                Box::pin(async move { Ok(Some(Type::$pg_ty)) })
            }
        }

        $(impl ToSql<$asp_ty, Pg> for $rust_ty {
            fn to_sql<'a>(
                &'a self,
                _metadata: &Option<Type>,
                collector: &'a mut <Pg as Backend>::BindCollector,
            ) -> AnyResult<<Pg as Backend>::RawValue<'a>> {
                PgToSql::to_sql(self, &Type::$pg_ty, collector.buffer())?;

                Ok(&[])
            }
        })+
    )+};
}

delegate_to_pgtosql! {
    bool => Bool => BOOL;
    i8 => TinyInt => CHAR;
    i16 => SmallInt => INT2;
    i32 => Integer => INT4;
    i64 => BigInt => INT8;
    f32 => Float => FLOAT4;
    f64 => Double => FLOAT8;
    String => Text => TEXT;
    Vec<u8>, &'_ [u8] => Binary => BYTEA;
    uuid::Uuid => Uuid => UUID
}

macro_rules! delegate_to_pgfromsql {
    ($($($rust_ty: ty),+ => $asp_ty: ty => $pg_ty: ident);+) => {$(
        $(impl<'a> FromSql<'a, $asp_ty, Pg> for $rust_ty {
            fn from_sql(_metadata: &Option<Type>, raw: &'a [u8]) -> AnyResult<Self> {
                Ok(PgFromSql::from_sql(&Type::$pg_ty, raw)?)
            }
        })+
    )+};
}

delegate_to_pgfromsql! {
    bool => Bool => BOOL;
    i8 => TinyInt => CHAR;
    i16 => SmallInt => INT2;
    i32 => Integer => INT4;
    i64 => BigInt => INT8;
    f32 => Float => FLOAT4;
    f64 => Double => FLOAT8;
    String => Text => TEXT;
    Vec<u8> => Binary => BYTEA;
    uuid::Uuid => Uuid => UUID
}

impl<'a> FromSql<'a, Binary, Pg> for &'a [u8] {
    fn from_sql(_metadata: &Option<Type>, raw: &'a [u8]) -> AnyResult<Self> {
        Ok(PgFromSql::from_sql(&Type::BYTEA, raw)?)
    }
}

impl<'a> FromSql<'a, Text, Pg> for &'a str {
    fn from_sql(_metadata: &Option<Type>, raw: &'a [u8]) -> AnyResult<Self> {
        Ok(PgFromSql::from_sql(&Type::TEXT, raw)?)
    }
}
