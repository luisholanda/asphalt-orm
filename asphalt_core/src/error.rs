use std::backtrace::Backtrace;
use std::error::Error as StdError;
use std::fmt::Write;

/// A generic error.
pub type AnyError = Box<dyn StdError + Send + Sync + 'static>;

/// Result from a user defined function with an unknown error type.
pub type AnyResult<T> = Result<T, AnyError>;

/// Result from a query.
pub type QueryResult<T> = Result<T, Error>;

pub struct Error {
    kind: ErrorKind,
    backtrace: Backtrace,
}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }
}

pub enum ErrorKind {
    NotFound,
    DatabaseError(
        DatabaseErrorKind,
        Box<dyn DatabaseErrorInformation + Send + Sync>,
    ),
    QueryBuilderError(AnyError),
    DeserializationError(AnyError),
    RollbackTransaction,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum DatabaseErrorKind {
    UniqueViolation,
    ForeignKeyViolation,
    UnableToSendCommand,
    SerializationFailure,
}

pub trait DatabaseErrorInformation {
    fn message(&self) -> &str;
    fn details(&self) -> Option<&str>;
    fn hint(&self) -> Option<&str>;
    fn table(&self) -> Option<&str>;
    fn column(&self) -> Option<&str>;
    fn constraint(&self) -> Option<&str>;
}

impl std::fmt::Debug for dyn DatabaseErrorInformation + Send + Sync {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.message())
    }
}
