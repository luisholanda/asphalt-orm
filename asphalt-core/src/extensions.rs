use std::convert::Infallible;
/// Type that resolves to `Ty` if `Db` has support for `Ext` (i.e. it implements `Has<Ext>`),
/// otherwise, resolves to `()`.
///
/// This effectively removes the field from the compiled code if the extension isn't supported.
///
/// This is only effective if `Ty` implements `Default`, otherwise you will not be able to
/// set the field value.
pub type OrUnit<Db, Ext, Ty> = <(Db, Ty) as ImplOrRemove<Ext>>::Field;

/// Type that resolves to `Ty` if `Db` has support for `Ext` (i.e. it implements `Has<Ext>`),
/// otherwise, resolves to `!`.
///
/// This effectively removes the variant from the compiled code if the extension isn't supported.
pub type OrNever<Db, Ext, Ty> = <(Db, Ty) as ImplOrRemove<Ext>>::Variant;

#[doc(hidden)]
pub trait ImplOrRemove<Ext> {
    type Variant;
    type Field;
}

default impl<T, Ext> ImplOrRemove<Ext> for T {
    type Variant = Infallible;
    type Field = ();
}

/// Marker trait telling a backend supports the `Ext` extension.
pub trait Supports<Ext> {}

#[doc(hidden)]
pub trait IfSupport<Ext> {
    const SUPPORT: bool = false;
}

impl<T, Ext> IfSupport<Ext> for T {
    default const SUPPORT: bool = false;
}

impl<T, Ext> IfSupport<Ext> for T
where
    T: Supports<Ext>,
{
    const SUPPORT: bool = true;
}

/// Returns whether the backend supports a given transaction.
///
/// Returns `true` only when `Db: Supports<Ext>`.
pub const fn supports<Db, Ext>() -> bool
where
    Db: IfSupport<Ext>,
{
    <Db as IfSupport<Ext>>::SUPPORT
}

/// Define a new backend extension type.
///
/// An extension type is a marker type representing a specific feature which may
/// not be supported by all backends.
///
/// Be aware, not all backends are necessary SQL databases, and they can support
/// only a very small subset of the language (e.g. cassandra with it CQL).
#[macro_export]
macro_rules! define_extension {
    ($(#[$meta:  meta])* $ext: ident) => {
        $(#[$meta])*
        pub struct $ext;

        impl<U, T> ImplOrRemove<$ext> for (T, U)
        where
            T: Supports<$ext>
        {
            type Variant = U;
            type Field = U;
        }
    };
}

define_extension! {
    /// Extension showing that the backend has support for `UNION`/`INTERSECT`/`EXCLUDE`.
    Union
}

define_extension! {
    /// Extension showing that the backend has support for transactions.
    Transaction
}

define_extension! {
    /// Extension showing that the backend has support for transaction isolation levels.
    IsolationLevel
}

define_extension! {
    /// Extension showing that the backend has support for the `READ ONLY` transaction mode.
    ReadOnly
}
