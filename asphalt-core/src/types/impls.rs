macro_rules! __define_aliases {
    ($($alias_ty: ident)+, $sql_ty: ty, $name: expr) => {
        $(
            #[doc = "Alias to `"]
            #[doc = $name]
            #[doc = "`"]
            pub type $alias_ty = $sql_ty;
        )+
    };
}

macro_rules! define_sql_types {
    ($($sql_name: literal $sql_ty: ident $(aliased as $($alias_ty: ident)*)?),+,) => {
        $(
            #[doc = "The `"]
            #[doc = $sql_name]
            #[doc = "` SQL type."]
            pub struct $sql_ty;
            
            impl $crate::types::NotNull for $sql_ty {}

            $(__define_aliases!($($alias_ty)+, $sql_ty, stringify!($sql_ty));)?
        )*
    };
}

define_sql_types! {
    "BIGINT" BigInt aliased as BigSerial,
    "BINARY" Binary,
    "BOOL" Bool,
    "DATE" Date,
    "DOUBLE" Double,
    "FLOAT" Float,
    "INTEGER" Integer aliased as Serial,
    "INTERVAL" Interval,
    "NUMERIC" Numeric aliased as Decimal,
    "SMALLINT" SmallInt aliased as SmallSerial,
    "TEXT" Text aliased as VarChar,
    "TIME" Time,
    "TIMESTAMP" Timestamp,
    "TINYINT" TinyInt,
    "TIMESTAMPTZ" TimestampTz,
    "UUID" Uuid,
    "JSON" Json,
}

/// The `ARRAY` SQL type.
pub struct Array<SqlTy>(SqlTy);
