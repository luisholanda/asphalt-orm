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

            $(__define_aliases!($($alias_ty)+, $sql_ty, stringify!($sql_ty));)?
        )*
    };
}

define_sql_types! {
    "BIGINT" BigInt aliased as BigSerial,
    "BINARY" Binary,
    "BOOL" BOOL,
    "DATE" DATE,
    "DOUBLE" DOUBLE,
    "FLOAT" FLOAT,
    "INTEGER" Integer aliased as Serial,
    "INTERVAL" INTERVAL,
    "NUMERIC" Numeric aliased as Decimal,
    "SMALLINT" SmallInt aliased as SmallSerial,
    "TEXT" Text aliased as VarChar,
    "TIME" TIME,
    "TIMESTAMP" TIMESTAMP,
    "TINYINT" TINYINT,
    "TIMESTAMPTZ" TIMESTAMPTZ,
    "UUID" UUID,
    "JSON" JSON,
}

/// The `ARRAY` SQL type.
pub struct Array<SqlTy>(SqlTy);
