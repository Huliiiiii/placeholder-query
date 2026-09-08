use placeholder_query_core::types::Ident;
use std::marker::PhantomData;

use super::expr;

pub trait Mode {
    type Field<T>;
}

pub struct Value;

impl Mode for Value {
    type Field<T> = T;
}

pub struct Expr;

impl Mode for Expr {
    type Field<T> = expr::Expr<T>;
}

#[derive_where::derive_where(Clone)]
pub struct Name<T = ()>(pub Ident, PhantomData<fn() -> T>);

impl Mode for Name {
    type Field<T> = Name<T>;
}

impl<T> Name<T> {
    pub fn new(name: impl Into<Ident>) -> Self {
        Self(name.into(), PhantomData)
    }
}

impl<T> std::fmt::Debug for Name<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
