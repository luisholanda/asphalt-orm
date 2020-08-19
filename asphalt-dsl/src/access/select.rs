use super::Access;
use crate::expressions::{Condition, IsExpression, PredicateOn, SqlTypeOf};
use crate::schemas::{IsTable, AllColumns};
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::types::Bool;
use std::marker::PhantomData;

/// A `SELECT` SQL query.
///
/// This type have three generic parameters, in order:
///
/// * the database backend;
/// * the SQL context (a table or a set of joins); and
/// * the current selection.
pub struct Select<'a, Db, T, Sel>
where
    Sel: IsExpression,
    Db: Backend + HasSqlType<Bool> + HasSqlType<SqlTypeOf<Sel>>,
{
    access: &'a Access<Db>,
    selection: Sel,
    where_clause: Condition<'a, Db>,
    _phantom: PhantomData<*mut T>,
}

impl<'a, Db, T> Select<'a, Db, T, AllColumns<T>>
where
    Db: Backend + HasSqlType<Bool> + HasSqlType<SqlTypeOf<AllColumns<T>>>,
    T: IsTable,
{
    pub(crate) fn from_table(access: &'a Access<Db>) -> Self {
        Select {
            access,
            selection: Default::default(),
            where_clause: Condition::r#true(),
            _phantom: PhantomData,
        }
    }
}

impl<'a, Db, T, Sel> Select<'a, Db, T, Sel>
where
    Sel: IsExpression,
    Db: Backend + HasSqlType<Bool> + HasSqlType<SqlTypeOf<Sel>>,
    T: IsTable,
{
    /// Filter the current query with the given predicate.
    ///
    /// Effectively does an `AND` of the current predicate with
    /// the new predicate.
    pub fn filter<'b, P>(self, predicate: P) -> Select<'b, Db, T, Sel>
    where
        P: PredicateOn<'b, Db, T> + 'b,
        Db: HasSqlType<Bool>,
        'a: 'b,
    {
        Select {
            access: self.access,
            selection: self.selection,
            where_clause: self.where_clause.and(predicate.as_expression().into()),
            _phantom: self._phantom,
        }
    }

    /// Filter the current query with the also given predicate.
    ///
    /// Effectively does an `OR` of the current predicate with
    /// the new predicate.
    pub fn or_filter<'b, P>(self, predicate: P) -> Select<'b, Db, T, Sel>
    where
        P: PredicateOn<'b, Db, T> + 'b,
        Db: HasSqlType<Bool>,
        'a: 'b,
    {
        Select {
            access: self.access,
            selection: self.selection,
            where_clause: self.where_clause.or(predicate.as_expression().into()),
            _phantom: self._phantom,
        }
    }
}
