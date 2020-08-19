use std::backtrace::Backtrace;
use std::error::Error as StdError;

/// A generic error.
pub type AnyError = Box<dyn StdError + Send + Sync + 'static>;

/// Result from a user defined function with an unknown error type.
pub type AnyResult<T> = Result<T, AnyError>;

/// Result from a query.
pub type QueryResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    backtrace: Option<Backtrace>,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use std::io::Write;
        match &self.kind {
            ErrorKind::NotFound => f.write_str("No row returned when one was expected"),
            ErrorKind::DatabaseError(kind, info) => match kind {
                DatabaseErrorKind::UniqueViolation => {
                    write!(f, "Unique violation: {}", info.message())
                }
                DatabaseErrorKind::ForeignKeyViolation => {
                    write!(f, "Foreign key violation: {}", info.message())
                }
                DatabaseErrorKind::SerializationFailure => {
                    write!(f, "Serialization failure: {}", info.message())
                }
                DatabaseErrorKind::ReadOnlyTransaction => {
                    write!(f, "Tried to write in a RO-transaction: {}", info.message())
                }
                DatabaseErrorKind::Unknown => write!(f, "Unknown error: {}", info.message()),
            },
            ErrorKind::DeserializationError(err) => {
                write!(f, "Error while deserializing value: {}", err)
            }
            ErrorKind::QueryBuilderError(err) => write!(f, "Error while building query: {}", err),
            ErrorKind::SerializationError(err) => {
                write!(f, "Error while serializing value: {}", err)
            }
            ErrorKind::RollbackTransaction => write!(f, "Transaction rollback"),
        }
    }
}

impl StdError for Error {}

impl Error {
    pub fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    pub fn deserialization_failure(error: AnyError) -> Self {
        let backtrace = error.backtrace().is_none().then(Backtrace::capture);

        Self {
            kind: ErrorKind::DeserializationError(error),
            backtrace,
        }
    }

    pub fn serialization_failure(error: AnyError) -> Self {
        let backtrace = error.backtrace().is_none().then(Backtrace::capture);

        Self {
            kind: ErrorKind::SerializationError(error),
            backtrace,
        }
    }

    pub fn database_error<Info>(kind: DatabaseErrorKind, info: Info) -> Self
    where
        Info: DatabaseErrorInformation + Send + Sync + 'static,
    {
        Self {
            kind: ErrorKind::DatabaseError(kind, Box::new(info)),
            backtrace: Some(Backtrace::capture()),
        }
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    NotFound,
    DatabaseError(
        DatabaseErrorKind,
        Box<dyn DatabaseErrorInformation + Send + Sync>,
    ),
    QueryBuilderError(AnyError),
    DeserializationError(AnyError),
    SerializationError(AnyError),
    RollbackTransaction,
}

impl ErrorKind {
    pub(crate) fn is_serialization_failure(&self) -> bool {
        match self {
            Self::DatabaseError(DatabaseErrorKind::SerializationFailure, _) => true,
            _ => false,
        }
    }

    pub(crate) fn is_read_only_transaction(&self) -> bool {
        match self {
            Self::DatabaseError(DatabaseErrorKind::ReadOnlyTransaction, _) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[non_exhaustive]
pub enum DatabaseErrorKind {
    UniqueViolation,
    ForeignKeyViolation,
    SerializationFailure,
    ReadOnlyTransaction,
    Unknown,
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

impl DatabaseErrorInformation for String {
    fn message(&self) -> &str {
        self
    }

    fn details(&self) -> Option<&str> {
        None
    }

    fn hint(&self) -> Option<&str> {
        None
    }

    fn table(&self) -> Option<&str> {
        None
    }

    fn column(&self) -> Option<&str> {
        None
    }

    fn constraint(&self) -> Option<&str> {
        None
    }
}
