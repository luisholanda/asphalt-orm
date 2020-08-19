use crate::expressions::{Condition, PredicateOn, SqlTypeOf};
use crate::schemas::IsTable;
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::connection::Connection;
use asphalt_core::types::Bool;
use std::marker::PhantomData;

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
    pub fn from<T: IsTable>(&self, table: T) -> FromTable<'_, Db, T>
    where
        Db: HasSqlType<SqlTypeOf<T::AllColumns>>,
    {
        drop(table);
        FromTable {
            access: self,
            _phantom: PhantomData,
        }
    }
}

pub struct FromTable<'a, Db: Backend, T> {
    access: &'a Access<Db>,
    _phantom: PhantomData<*mut T>,
}

impl<'a, Db: Backend, T: IsTable> FromTable<'a, Db, T> {
    pub fn filter<'b, P>(self, predicate: P) -> Filter<'b, Db, T>
    where
        P: PredicateOn<'b, Db, T> + 'b,
        Db: HasSqlType<Bool>,
        'a: 'b,
    {
        Filter {
            access: self.access,
            predicate: WhereClause {
                clause: predicate.as_expression().into(),
            },
            _phantom: self._phantom,
        }
    }
}

pub struct Filter<'a, Db: Backend + HasSqlType<Bool>, T> {
    access: &'a Access<Db>,
    predicate: WhereClause<'a, Db>,
    _phantom: PhantomData<*mut T>,
}

struct WhereClause<'a, Db: Backend + HasSqlType<Bool>> {
    clause: Condition<'a, Db>,
}
