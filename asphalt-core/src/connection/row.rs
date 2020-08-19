use super::RawConnection;
use crate::backend::{Backend, HasSqlType};
use crate::error::{AnyResult, QueryResult};
use crate::types::FromSql;
use futures_util::stream::BoxStream;

/// A stream of rows resulting from the execution of a query by a connection `Conn`.
pub type RowStream<'c, Conn> = BoxStream<'c, QueryResult<<Conn as RawConnection>::Row>>;

/// A row of data returned from the database backend.
pub trait Row {
    /// The backend from which this row can be returned.
    type Backend: Backend;

    /// Number of columns in this row.
    fn n_columns(&self) -> usize;

    /// get a column using a specific rust type.
    fn get_column<'a, SqlTy, RustTy>(&'a self, idx: usize) -> AnyResult<RustTy>
    where
        Self::Backend: HasSqlType<SqlTy>,
        RustTy: FromSql<'a, SqlTy, Self::Backend>;
}
