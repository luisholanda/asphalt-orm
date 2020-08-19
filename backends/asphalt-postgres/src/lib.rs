#![feature(generic_associated_types)]
use asphalt_core::backend::{Backend, TypeMetadata};
use asphalt_core::error::{DatabaseErrorInformation, DatabaseErrorKind, Error};
use asphalt_core::values::RawValue;
use std::error::Error as StdError;
use tokio_postgres::error::SqlState;
use tokio_postgres::types::Type;

mod connection;
mod metadata;
mod query;
mod types;

#[doc(inline)]
pub use self::connection::PgRawConnection;
#[doc(inline)]
pub use self::metadata::MetadataLookup;
#[doc(inline)]
pub use self::query::{PgBindCollector, PgQuery, PgQueryWriter};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct Pg;

impl Backend for Pg {
    type Query = PgQuery;
    type QueryWriter = PgQueryWriter;
    type BindName = (u16, Option<Type>);
    type BindCollector = PgBindCollector;
    type RawConnection = PgRawConnection;
    type RawValue<'a> = &'a [u8];
}

impl TypeMetadata for Pg {
    // The metadata is handled automatically by the database driver.
    type TypeMetadata = Option<Type>;
    type MetadataLookup = MetadataLookup;
}

impl RawValue<Pg> for &'_ [u8] {
    fn is_null(&self) -> bool {
        self.is_empty()
    }

    fn null_value() -> Self {
        &[]
    }
}

pub(crate) fn dberror_to_query_error(err: tokio_postgres::error::DbError) -> Error {
    let kind = if *err.code() == SqlState::UNIQUE_VIOLATION {
        DatabaseErrorKind::UniqueViolation
    } else if *err.code() == SqlState::FOREIGN_KEY_VIOLATION {
        DatabaseErrorKind::ForeignKeyViolation
    } else if *err.code() == SqlState::READ_ONLY_SQL_TRANSACTION {
        DatabaseErrorKind::ReadOnlyTransaction
    } else if [
        SqlState::INVALID_JSON_TEXT,
        SqlState::INVALID_XML_DOCUMENT,
        SqlState::INVALID_XML_COMMENT,
        SqlState::INVALID_XML_CONTENT,
    ]
    .contains(err.code())
    {
        DatabaseErrorKind::SerializationFailure
    } else {
        DatabaseErrorKind::Unknown
    };

    Error::database_error(kind, PgErrorInfo(err))
}

pub(crate) fn error_to_query_error(err: tokio_postgres::Error) -> Error {
    if let Some(db_error) = err
        .source()
        .and_then(|err| err.downcast_ref::<tokio_postgres::error::DbError>())
        .cloned()
    {
        dberror_to_query_error(db_error)
    } else {
        Error::database_error(DatabaseErrorKind::Unknown, err.to_string())
    }
}

pub struct PgErrorInfo(tokio_postgres::error::DbError);

impl DatabaseErrorInformation for PgErrorInfo {
    fn message(&self) -> &str {
        self.0.message()
    }

    fn details(&self) -> Option<&str> {
        self.0.detail()
    }

    fn hint(&self) -> Option<&str> {
        self.0.hint()
    }

    fn table(&self) -> Option<&str> {
        self.0.table()
    }

    fn column(&self) -> Option<&str> {
        self.0.column()
    }

    fn constraint(&self) -> Option<&str> {
        self.0.constraint()
    }
}
