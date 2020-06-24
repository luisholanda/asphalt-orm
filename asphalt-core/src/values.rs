use crate::backend::Backend;

/// Trait for the raw value for a given backend.
///
/// Is important to note that type implementing this trait doesn't need to own
/// the value of the data itself, they can instead be used to fetch the data
/// from inside a `BindCollector` or `RowExtractor`. This will improve the
/// performance of the backend implementation as values' data can be allocated
/// in a continuous region of memory.
pub trait RawValue<Db: Backend>: Sized {
    /// Is this value the `NULL` value?
    fn is_null(&self) -> bool;

    /// Returns the null value for this backend.
    fn null_value() -> Self;
}
