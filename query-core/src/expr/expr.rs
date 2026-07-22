use std::{marker::PhantomData, ops::Add};

use derive_where::derive_where;

use crate::backend::QueryBackend;

use super::ColumnRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExprId(pub(crate) usize);

impl Add<usize> for ExprId {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive_where(
    Clone,
    Debug,
    PartialEq;
    <B as QueryBackend>::BinaryOp,
    <B as QueryBackend>::UnaryOp,
    <B as QueryBackend>::Value
)]
pub enum ExprNode<B: QueryBackend> {
    Column(ColumnRef),
    Value(B::Value),
    Values(Vec<B::Value>),
    Unary {
        op: B::UnaryOp,
        expr: ExprId,
    },
    Binary {
        op: B::BinaryOp,
        left: ExprId,
        right: ExprId,
    },
}

impl<B: QueryBackend> ExprNode<B> {
    fn shift_ids(self, offset: usize) -> Self {
        match self {
            Self::Unary { op, expr } => Self::Unary {
                op,
                expr: expr + offset,
            },
            Self::Binary { op, left, right } => Self::Binary {
                op,
                left: left + offset,
                right: right + offset,
            },
            expr => expr,
        }
    }
}

#[derive_where(Clone, Debug, PartialEq; ExprNode<B>)]
pub struct ExprArena<B: QueryBackend> {
    pub(crate) nodes: Vec<ExprNode<B>>,
    _backend: PhantomData<fn() -> B>,
}

impl<B: QueryBackend> ExprArena<B> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            _backend: PhantomData,
        }
    }

    pub(crate) fn push(&mut self, expr: ExprNode<B>) -> ExprId {
        let id = ExprId(self.nodes.len());
        self.nodes.push(expr);
        id
    }

    pub fn get(&self, id: ExprId) -> &ExprNode<B> {
        &self.nodes[id.0]
    }

    pub fn append(&mut self, fragment: Expr<B>) -> ExprId {
        let offset = self.nodes.len();
        let root = fragment.root + offset;

        self.nodes.extend(
            fragment
                .exprs
                .nodes
                .into_iter()
                .map(|expr| expr.shift_ids(offset)),
        );

        root
    }
}

#[derive_where(Clone, Debug, PartialEq; ExprArena<B>)]
pub struct Expr<B: QueryBackend> {
    exprs: ExprArena<B>,
    root: ExprId,
}

impl<B: QueryBackend> Expr<B> {
    fn from_node(expr: ExprNode<B>) -> Self {
        let mut exprs = ExprArena::new();
        let root = exprs.push(expr);

        Self { exprs, root }
    }

    pub fn value(value: B::Value) -> Self {
        Self::from_node(ExprNode::Value(value))
    }

    pub fn unary(op: B::UnaryOp, expr: impl Into<Self>) -> Self {
        let expr = expr.into();
        let mut exprs = ExprArena::new();
        let expr = exprs.append(expr);
        let root = exprs.push(ExprNode::Unary { op, expr });

        Self { exprs, root }
    }

    pub fn binary(op: B::BinaryOp, left: Self, right: impl Into<Self>) -> Self {
        let right = right.into();
        let mut exprs = ExprArena::new();
        let left = exprs.append(left);
        let right = exprs.append(right);
        let root = exprs.push(ExprNode::Binary { op, left, right });

        Self { exprs, root }
    }

    pub fn values(values: impl IntoIterator<Item = impl Into<B::Value>>) -> Self {
        Self::from_node(ExprNode::Values(
            values.into_iter().map(Into::into).collect(),
        ))
    }
}

impl<B> From<ExprNode<B>> for Expr<B>
where
    B: QueryBackend,
{
    fn from(expr: ExprNode<B>) -> Self {
        Self::from_node(expr)
    }
}
