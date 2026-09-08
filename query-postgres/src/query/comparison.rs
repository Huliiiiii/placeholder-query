use super::{
    Expr,
    params::{Param, ParamType},
};

#[doc(hidden)]
pub enum ComparisonArg<T> {
    Scalar(Expr<T>),
    Any(Expr<Vec<T>>),
    All(Expr<Vec<T>>),
}

pub struct Any<T>(Expr<Vec<T>>);

pub struct All<T>(Expr<Vec<T>>);

impl<T> Any<T> {
    pub(crate) fn new(values: impl Into<Expr<Vec<T>>>) -> Self {
        Self(values.into())
    }
}

impl<T> All<T> {
    pub(crate) fn new(values: impl Into<Expr<Vec<T>>>) -> Self {
        Self(values.into())
    }
}

impl<T> From<Expr<T>> for ComparisonArg<T> {
    fn from(expr: Expr<T>) -> Self {
        Self::Scalar(expr)
    }
}

impl<T: ParamType> From<Param<T>> for ComparisonArg<T> {
    fn from(param: Param<T>) -> Self {
        Self::from(Expr::from(param))
    }
}

impl<T> From<Any<T>> for ComparisonArg<T> {
    fn from(any: Any<T>) -> Self {
        Self::Any(any.0)
    }
}

impl<T> From<All<T>> for ComparisonArg<T> {
    fn from(all: All<T>) -> Self {
        Self::All(all.0)
    }
}
