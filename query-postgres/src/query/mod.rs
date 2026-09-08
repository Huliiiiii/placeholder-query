pub mod columns;
pub mod comparison;
mod expr;
pub mod mode;
pub(crate) mod operator;
mod order;
pub mod params;
pub mod projection;
pub mod relation;
pub mod select;
pub mod table;

pub use expr::Expr;
pub use operator::{BinaryOp, UnaryOp};
pub use order::OrderExpr;
pub use placeholder_query_core::types::Ident;
pub use projection::{MappedProjection, Projection, ProjectionExt};

#[doc(hidden)]
pub enum Erased {}

pub fn any<T>(values: impl Into<Expr<Vec<T>>>) -> comparison::Any<T> {
    comparison::Any::new(values)
}

pub fn all<T>(values: impl Into<Expr<Vec<T>>>) -> comparison::All<T> {
    comparison::All::new(values)
}
