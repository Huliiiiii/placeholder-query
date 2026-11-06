use std::{fmt, marker::PhantomData, sync::Arc};

use rel9_core::types::ParamId;

use super::{
    Erased,
    comparison::ComparisonArg,
    operator::{BinaryOp, UnaryOp},
    params::{Param, ParamType, SqlType},
    relation::RelationId,
};

#[derive(Debug, PartialEq)]
pub(crate) enum ExprNode {
    Boolean(bool),
    Column {
        relation: RelationId,
        field: u16,
    },
    Param {
        param_id: ParamId,
        sql_type: SqlType,
    },
    Unary {
        op: UnaryOp,
        expr: Expr,
    },
    Binary {
        op: BinaryOp,
        left: Expr,
        right: Expr,
    },
}

#[derive_where::derive_where(Clone, PartialEq)]
pub struct Expr<T = Erased>(pub(crate) Arc<ExprNode>, PhantomData<fn() -> T>);

impl<T> fmt::Debug for Expr<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Expr<T> {
    pub(crate) fn from_node(node: ExprNode) -> Self {
        Self(Arc::new(node), PhantomData)
    }

    #[doc(hidden)]
    pub fn erase(self) -> Expr {
        self.cast()
    }

    pub(crate) fn cast<U>(self) -> Expr<U> {
        Expr(self.0, PhantomData)
    }

    fn binary(op: BinaryOp, left: Expr, right: Expr) -> Self {
        Self::from_node(ExprNode::Binary { op, left, right })
    }

    pub(crate) fn map_columns(
        self,
        map: &mut (impl FnMut(RelationId, u16) -> (RelationId, u16) + ?Sized),
    ) -> Self {
        let node = match &*self.0 {
            ExprNode::Boolean(_) => return self,
            ExprNode::Column {
                relation: relation_id,
                field,
            } => {
                let (relation, field) = map(*relation_id, *field);
                ExprNode::Column { relation, field }
            }
            ExprNode::Param { .. } => return self,
            ExprNode::Unary { op, expr } => ExprNode::Unary {
                op: op.clone(),
                expr: expr.clone().map_columns(map),
            },
            ExprNode::Binary { op, left, right } => ExprNode::Binary {
                op: op.clone(),
                left: left.clone().map_columns(map),
                right: right.clone().map_columns(map),
            },
        };
        Self::from_node(node)
    }

    fn compare(
        self,
        scalar: BinaryOp,
        any: BinaryOp,
        all: BinaryOp,
        right: impl Into<ComparisonArg<T>>,
    ) -> Expr<bool> {
        let (op, right): (BinaryOp, Expr) = match right.into() {
            ComparisonArg::Scalar(right) => (scalar, right.erase()),
            ComparisonArg::Any(right) => (any, right.erase()),
            ComparisonArg::All(right) => (all, right.erase()),
        };
        Expr::binary(op, self.erase(), right)
    }

    pub fn eq(self, right: impl Into<ComparisonArg<T>>) -> Expr<bool> {
        self.compare(BinaryOp::Eq, BinaryOp::EqAny, BinaryOp::EqAll, right)
    }

    pub fn gt(self, right: impl Into<ComparisonArg<T>>) -> Expr<bool> {
        self.compare(BinaryOp::Gt, BinaryOp::GtAny, BinaryOp::GtAll, right)
    }

    pub fn gte(self, right: impl Into<ComparisonArg<T>>) -> Expr<bool> {
        self.compare(BinaryOp::Gte, BinaryOp::GteAny, BinaryOp::GteAll, right)
    }
}

impl Expr<bool> {
    pub(crate) fn true_literal() -> Self {
        Self::from_node(ExprNode::Boolean(true))
    }

    pub(crate) fn is_not_null(self) -> Self {
        Self::unary(UnaryOp::IsNotNull, self)
    }

    pub fn and(self, right: Self) -> Self {
        Self::binary(BinaryOp::And, self.erase(), right.erase())
    }

    pub fn or(self, right: Self) -> Self {
        Self::binary(BinaryOp::Or, self.erase(), right.erase())
    }

    pub fn not(self) -> Self {
        Self::unary(UnaryOp::Not, self)
    }

    fn unary(op: UnaryOp, expr: Self) -> Self {
        Self::from_node(ExprNode::Unary {
            op,
            expr: expr.erase(),
        })
    }
}

impl Expr<String> {
    pub fn like(self, pattern: impl Into<ComparisonArg<String>>) -> Expr<bool> {
        self.compare(
            BinaryOp::Like,
            BinaryOp::LikeAny,
            BinaryOp::LikeAll,
            pattern,
        )
    }
}

impl<T: ParamType> From<Param<T>> for Expr<T> {
    fn from(param: Param<T>) -> Self {
        Self::from_node(ExprNode::Param {
            param_id: param.id,
            sql_type: T::ty(),
        })
    }
}
