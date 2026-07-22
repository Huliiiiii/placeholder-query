use placeholder_query_core::expr::Expr as CoreExpr;

use crate::{backend::Pg, value::Value};

use super::{
    column::Column,
    operator::{BinaryOp, UnaryOp},
};

#[derive(Clone, Debug, PartialEq)]
pub struct Expr(CoreExpr<Pg>);

impl Expr {
    pub fn and(self, right: Self) -> Self {
        Self(CoreExpr::binary(BinaryOp::And, self.0, right.0))
    }

    pub fn or(self, right: Self) -> Self {
        Self(CoreExpr::binary(BinaryOp::Or, self.0, right.0))
    }

    pub fn not(self) -> Self {
        Self(CoreExpr::unary(UnaryOp::Not, self.0))
    }
}

impl From<Value> for Expr {
    fn from(value: Value) -> Self {
        Self(CoreExpr::value(value))
    }
}

impl From<CoreExpr<Pg>> for Expr {
    fn from(value: CoreExpr<Pg>) -> Self {
        Self(value)
    }
}

impl From<Expr> for CoreExpr<Pg> {
    fn from(value: Expr) -> Self {
        value.0
    }
}

impl<T> From<Column<T>> for Expr {
    fn from(value: Column<T>) -> Self {
        Self(value.into())
    }
}

macro_rules! impl_from_value_type_for_expr {
    ($($ty:ty),* $(,)?) => {
        $(
            impl From<$ty> for Expr {
                fn from(value: $ty) -> Self {
                    Self::from(Value::from(value))
                }
            }
        )*
    };
}

impl_from_value_type_for_expr!(
    i8,
    i16,
    i32,
    i64,
    u8,
    u16,
    u32,
    f32,
    f64,
    &str,
    String,
    Vec<u8>,
);
