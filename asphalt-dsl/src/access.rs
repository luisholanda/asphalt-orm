use crate::expressions::SqlTypeOf;
use crate::schemas::{IsTable, AllColumns};
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::connection::Connection;
use asphalt_core::types::Bool;

mod select;
#[doc(inline)]
pub use self::select::Select;

pub struct Access<Db: Backend> {
    conn: Connection<Db>,
}

//
// let conn = pool.get().await?;
// let all_users = conn.from(users::table).await?;
// let users = conn.from(users::table).filter(..).await?;
// let user = conn.from(users::table).get_one(id).await?;
// conn.update(users::table).set(..).filter(..).await?;
// conn.insert_into(users::table).values(..).await?;
// [user_id.eq(id): BoolOp<user_id, _, {Eq}>]
//
impl<Db: Backend> Access<Db> {
    /// Create a `SELECT` query from the provided table.
    pub fn from<T: IsTable>(&self) -> Select<'_, Db, T, AllColumns<T>>
    where
        Db: HasSqlType<Bool> + HasSqlType<SqlTypeOf<T::AllColumns>>,
    {
        Select::from_table(self)
    }
}
