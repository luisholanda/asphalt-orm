use crate::schemas::{AppearsOnTable, IsTable};
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::types::{Bool, ToSql};

mod comparisons;
#[doc(inline)]
pub use self::comparisons::Condition;

/// A boxed SQL expression tree.
pub struct Expression<'a, Db: Backend + HasSqlType<SqlTy>, SqlTy> {
    expr: ExpressionTree<'a, Db, SqlTy>,
}

enum ExpressionTree<'a, Db: Backend + HasSqlType<SqlTy>, SqlTy> {
    Bound(Bound<'a, Db, SqlTy>),
}

/// Trait for types that represent a SQL expression.
pub trait IsExpression {
    /// The SQL type of the expression.
    type Type;
}

impl<T> IsExpression for &'_ T
where
    T: IsExpression,
{
    type Type = T::Type;
}

/// Convenience type alias for the SQL type of an expression.
pub type SqlTypeOf<T> = <T as IsExpression>::Type;

/// A type that can be converted to an expression of a given SQL type.
pub trait AsExpression<'a, SqlTy> {
    /// The expression that this type can be converted.
    type Expression: IsExpression<Type = SqlTy> + 'a;

    /// Convert a value to an expression.
    fn as_expression(self) -> Self::Expression;
}

/// Any expression can be converted to itself.
impl<'a, Ty> AsExpression<'a, Ty::Type> for Ty
where
    Ty: IsExpression + 'a
{
    type Expression = Self;

    fn as_expression(self) -> Self::Expression {
        self
    }
}

/// Marker trait for types that represent a predicate in a context.
///
/// Context here can mean a table (in a simple query) or a set of
/// tables (in case of a joined query).
///
/// The trait is used to prevent that queries that use non-existent
/// columns compile.
#[marker]
pub trait PredicateOn<'a, Db: Backend + HasSqlType<Bool> + 'a, T>:
    AsExpression<'a, Bool, Expression: Into<Condition<'a, Db>>>
{
}

/// If:
///
/// * The context is a single table; and
/// * The expression appears on the table; and
/// * The expression can be converted to a boolean SQL expression.
///
/// Then the expression is a predicate in the table.
impl<'a, E, T, Db> PredicateOn<'a, Db, T> for E
where
    T: IsTable,
    E: AppearsOnTable<T> + AsExpression<'a, Bool>,
    E::Expression: Into<Condition<'a, Db>>,
    Db: Backend + HasSqlType<Bool> + HasSqlType<SqlTypeOf<T::AllColumns>> + 'a,
{
}

/// The type for bound variables.
pub enum Bound<'a, Db: Backend + HasSqlType<SqlTy>, SqlTy> {
    /// We took the variable by reference.
    Ref(&'a dyn ToSql<Db, SqlTy>),
    /// We own the variable.
    Own(Box<dyn ToSql<Db, SqlTy>>)
}

impl<SqlTy, Ty, Db> AsExpression<'_, SqlTy> for Ty
where
    Ty: ToSql<SqlTy, Db>,
    Db: Backend + HasSqlType<SqlTy>,
{
}
