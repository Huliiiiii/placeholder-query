use std::marker::PhantomData;

use placeholder_query_builder::{
    expr::{ExprId, Exprs, Ident},
    value::Value,
};

use super::builder::PgQueryBuilder;

#[derive(Clone, Debug, PartialEq)]
pub struct PgQuery {
    pub sql: String,
    pub params: Vec<Value>,
}

#[derive(Clone, Debug)]
pub struct PgSelect<'a> {
    pub(crate) builder: &'a PgQueryBuilder,
    pub(crate) plan: PgQueryPlan,
}

impl PgSelect<'_> {
    pub fn build(self) -> PgQuery {
        self.builder.build(&self.plan)
    }
}

impl From<PgSelect<'_>> for PgQueryPlan {
    fn from(value: PgSelect<'_>) -> Self {
        value.plan
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PgQueryPlan {
    pub(crate) exprs: Exprs,
    pub(crate) from: PgTableRef,
    pub(crate) joins: Vec<PgJoin>,
    pub(crate) filters: Vec<ExprId>,
    pub(crate) select: Vec<ExprId>,
}

impl PgQueryPlan {
    pub(crate) fn new(from: PgTableRef) -> Self {
        Self {
            exprs: Exprs::new(),
            from,
            joins: Vec::new(),
            filters: Vec::new(),
            select: Vec::new(),
        }
    }

    pub fn build(&self, builder: &PgQueryBuilder) -> PgQuery {
        builder.build(self)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PgTableRef {
    pub(crate) name: Ident,
    pub(crate) alias: Ident,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PgJoin {
    pub(crate) table: PgTableRef,
    pub(crate) on: ExprId,
}

pub struct PgFrom<Refs> {
    pub(crate) plan: PgQueryPlan,
    pub(crate) refs: Refs,
    pub(crate) alias_count: usize,
}

pub struct PgSelectQuery<T, P> {
    pub(crate) plan: PgQueryPlan,
    pub(crate) _output: PhantomData<fn() -> (T, P)>,
}
