use std::ops::Add;

use crate::value::Value;

use super::Column;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExprId(pub(crate) usize);

impl Add<usize> for ExprId {
    type Output = Self;

    fn add(self, rhs: usize) -> Self::Output {
        Self(self.0 + rhs)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Column(Column),
    Value(Value),
    Values(Vec<Value>),
    Binary {
        op: BinaryOp,
        left: ExprId,
        right: ExprId,
    },
}

impl Expr {
    fn shift_ids(self, offset: usize) -> Self {
        match self {
            Self::Binary { op, left, right } => Self::Binary {
                op,
                left: left + offset,
                right: right + offset,
            },
            expr => expr,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Exprs {
    pub(crate) nodes: Vec<Expr>,
}

impl Exprs {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub(crate) fn push(&mut self, expr: Expr) -> ExprId {
        let id = ExprId(self.nodes.len());
        self.nodes.push(expr);
        id
    }

    pub fn get(&self, id: ExprId) -> &Expr {
        &self.nodes[id.0]
    }

    pub fn append(&mut self, fragment: ExprFragment) -> ExprId {
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

#[derive(Clone, Debug, PartialEq)]
pub struct ExprFragment {
    exprs: Exprs,
    root: ExprId,
}

impl ExprFragment {
    fn from_expr(expr: Expr) -> Self {
        let mut exprs = Exprs::new();
        let root = exprs.push(expr);

        Self { exprs, root }
    }

    pub(crate) fn binary(op: BinaryOp, left: Self, right: impl Into<Self>) -> Self {
        let right = right.into();
        let mut exprs = Exprs::new();
        let left = exprs.append(left);
        let right = exprs.append(right);
        let root = exprs.push(Expr::Binary { op, left, right });

        Self { exprs, root }
    }

    pub(crate) fn values(values: impl IntoIterator<Item = impl Into<Value>>) -> Self {
        Self::from_expr(Expr::Values(values.into_iter().map(Into::into).collect()))
    }
}

impl<T> From<T> for ExprFragment
where
    T: Into<Expr>,
{
    fn from(value: T) -> Self {
        Self::from_expr(value.into())
    }
}

impl IntoIterator for ExprFragment {
    type Item = ExprFragment;
    type IntoIter = std::iter::Once<ExprFragment>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    And,
    Or,
    Eq,
    In,
    Like,
}
