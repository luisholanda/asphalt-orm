use super::{Expression, IsExpression};
use asphalt_core::backend::{Backend, HasSqlType};
use asphalt_core::types::Bool;

/// An opaque SQL condition expression.
///
/// This is used to represent expression that MUST be a
/// boolean expression when rendered to SQL, e.g. a `WHERE`
/// in SQL queries.
pub struct Condition<'a, Db: Backend + HasSqlType<Bool>> {
    tree: ConditionTree<'a, Db>,
}

impl<Db: Backend + HasSqlType<Bool>> IsExpression for Condition<'_, Db> {
    type Type = Bool;
}

impl<'a, Db: Backend + HasSqlType<Bool>> Condition<'a, Db> {
    /// Create an always true condition.
    pub fn r#true() -> Self {
        Self {
            tree: ConditionTree::Lit(true)
        }
    }

    /// Create an always false condition.
    pub fn r#false() -> Self {
        Self {
            tree: ConditionTree::Lit(false)
        }
    }

    /// Does an `AND` of both conditions.
    pub fn and(self, other: Self) -> Self {
        use ConditionTree::*;
        Self {
            tree: match (self.tree, other.tree) {
                (Lit(true), right) => right,
                (Lit(false), _) => Lit(false),
                (left, Lit(true)) => left,
                (_, Lit(false)) => Lit(false),
                (And(mut left), And(mut right)) => {
                    left.append(&mut right);

                    And(left)
                }
                (And(mut left), right) => {
                    left.push(right);
                    And(left)
                }
                (left, And(mut right)) => {
                    // Ordering of conditions doesn't matter.
                    right.push(left);
                    And(right)
                }
                (left, right) => And(vec![left, right]),
            },
        }
    }

    /// Does an `OR` of both conditions.
    pub fn or(self, other: Self) -> Self {
        use ConditionTree::*;
        Self {
            tree: match (self.tree, other.tree) {
                (Lit(true), _) => Lit(true),
                (Lit(false), right) => right,
                (_, Lit(true)) => Lit(true),
                (left, Lit(false)) => left,
                (Or(mut left), Or(mut right)) => {
                    left.append(&mut right);

                    Or(left)
                }
                (Or(mut left), right) => {
                    left.push(right);
                    Or(left)
                }
                (left, Or(mut right)) => {
                    // Ordering of conditions doesn't matter.
                    right.push(left);
                    Or(right)
                }
                (left, right) => Or(vec![left, right]),
            },
        }
    }
}

// TODO: think in a way to group these allocations.
enum ConditionTree<'a, Db: Backend + HasSqlType<Bool>> {
    And(Vec<ConditionTree<'a, Db>>),
    Or(Vec<ConditionTree<'a, Db>>),
    Expr(Expression<'a, Db, Bool>),
    Lit(bool),
}
