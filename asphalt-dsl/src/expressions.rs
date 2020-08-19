use crate::schemas::{AppearsOnTable, IsTable};
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::types::{Bool, ToSql};

pub trait IsExpression {
    type Type;
}

impl<T> IsExpression for &'_ T
where
    T: IsExpression,
{
    type Type = T::Type;
}

pub type SqlTypeOf<T> = <T as IsExpression>::Type;

pub trait AsExpression<'a, SqlTy> {
    type Expression: IsExpression<Type = SqlTy> + 'a;

    fn as_expression(&self) -> Self::Expression;
}

impl<'a, Ty> AsExpression<'a, Ty::Type> for &'a Ty
where
    Ty: IsExpression + 'a,
{
    type Expression = Self;

    fn as_expression(&self) -> Self::Expression {
        self
    }
}

#[marker]
pub trait PredicateOn<'a, Db: Backend + HasSqlType<Bool> + 'a, T>:
    AsExpression<'a, Bool, Expression: Into<Condition<'a, Db>>>
{
}

impl<'a, E, T, Db> PredicateOn<'a, Db, T> for E
where
    T: IsTable,
    E: AppearsOnTable<T> + AsExpression<'a, Bool>,
    E::Expression: Into<Condition<'a, Db>>,
    Db: Backend + HasSqlType<Bool> + HasSqlType<SqlTypeOf<T::AllColumns>> + 'a,
{
}

pub struct Bound<'a, Db, SqlTy>(&'a dyn ToSql<SqlTy, Db>)
where
    Db: Backend + HasSqlType<SqlTy>;

pub struct Condition<'a, Db: Backend + HasSqlType<Bool>> {
    tree: ConditionTree<'a, Db>,
}

// TODO: think in a way to group these allocations.
enum ConditionTree<'a, Db: Backend + HasSqlType<Bool>> {
    And(Vec<ConditionTree<'a, Db>>),
    Or(Vec<ConditionTree<'a, Db>>),
    Expr(Expression<'a, Db, Bool>),
}

pub struct Expression<'a, Db: Backend + HasSqlType<SqlTy>, SqlTy> {
    expr: ExpressionTree<'a, Db, SqlTy>,
}

enum ExpressionTree<'a, Db: Backend + HasSqlType<SqlTy>, SqlTy> {
    Bound(Bound<'a, Db, SqlTy>),
}
