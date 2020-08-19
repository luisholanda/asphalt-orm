/// Market trait for extensions.
pub trait Extension {}

/// Marker trait telling a backend supports the `Ext` extension.
pub trait Supports<Ext: Extension> {}

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

        impl Extension for $ext {}
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
