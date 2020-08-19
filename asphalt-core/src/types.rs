mod from;
mod impls;
mod to;

#[doc(inline)]
pub use self::from::FromSql;
#[doc(inline)]
pub use self::impls::*;
#[doc(inline)]
pub use self::to::ToSql;

/// Marker trait for types that are not nullable.
pub trait NotNull {}

impl<T> !NotNull for Nullable<T> {}

/// Converts a type into its nullable version.
pub trait IntoNullable {
    /// The nullable version of this type.
    type Nullable;
}

impl<T> IntoNullable for T
where
    T: NotNull,
{
    type Nullable = Nullable<T>;
}

impl<T> IntoNullable for Nullable<T>
where
    T: NotNull + IntoNullable,
{
    type Nullable = T::Nullable;
}

/// A nullable SQL type.
///
/// By default, all types are assumed to be `NOT NULL`. This type wraps another one
/// indicating that this can be null.
pub struct Nullable<T: NotNull>(T);
