use super::RawConnection;
use crate::backend::Backend;
use crate::error::{AnyResult, QueryResult};
use std::borrow::Cow;
use futures_util::stream::BoxStream;

/// A stream of rows resulting from the execution of a query by a connection `Conn`.
pub type RowStream<'c, Conn> = BoxStream<'c, QueryResult<<Conn as RawConnection>::Row>>;

/// A row of data returned from the database backend.
pub trait Row {
    /// The backend from which this row can be returned.
    type Backend: Backend;

    /// Number of columns in this row.
    fn n_rows(&self) -> usize;
    /// get a column using a specific rust type.
    fn get_column<SqlTy, RustTy>(&self, idx: usize) -> AnyResult<RustTy>;
    /// Returns a description of the columns in this row.
    fn columns(&self) -> Cow<'_, [RowColumn<'_, Self::Backend>]>;
}

/// Dynamic information about a [`Row`] column.
pub struct RowColumn<'r, Db: Backend> {
    name: Cow<'r, str>,
    table: Cow<'r, str>,
    schema: Cow<'r, str>,
    r#type: Db::TypeMetadata,
}

impl<Db: Backend> Clone for RowColumn<'_, Db> {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            table: self.table.clone(),
            schema: self.schema.clone(),
            r#type: self.r#type.clone()
        }
    }

    fn clone_from(&mut self, other: &Self) {
        self.name.clone_from(&other.name);
        self.table.clone_from(&other.table);
        self.schema.clone_from(&other.schema);
        self.r#type.clone_from(&other.r#type);
    }
}

impl<Db: Backend> RowColumn<'_, Db> {
    /// The name of this column.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The table of this column.
    pub fn table(&self) -> &str {
        &self.table
    }

    /// The schema of this column.
    pub fn schema(&self) -> &str {
        &self.schema
    }

    /// The type of this column.
    pub fn type_(&self) -> &Db::TypeMetadata {
        &self.r#type
    }
}
