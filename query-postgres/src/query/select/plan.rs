use placeholder_query_core::{
    expr::{ExprArena, ExprId},
    ident::Ident,
};

use crate::{backend::Pg, statement::PgStatement};

#[derive(Clone, Debug)]
pub struct PgSelect<P> {
    pub(crate) plan: PgSelectPlan,
    pub(crate) _projection: P,
}

impl<P> PgSelect<P> {
    pub fn build(self) -> PgStatement {
        Pg.build(&self.plan)
    }
}

impl<P> From<PgSelect<P>> for PgSelectPlan {
    fn from(value: PgSelect<P>) -> Self {
        value.plan
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PgSelectPlan {
    pub(crate) exprs: ExprArena<Pg>,
    pub(crate) from: PgTableRef,
    pub(crate) joins: Vec<PgJoin>,
    pub(crate) filters: Vec<ExprId>,
    pub(crate) select: Vec<ExprId>,
}

impl PgSelectPlan {
    pub(crate) fn new(from: PgTableRef) -> Self {
        Self {
            exprs: ExprArena::new(),
            from,
            joins: Vec::new(),
            filters: Vec::new(),
            select: Vec::new(),
        }
    }

    pub fn build(&self, pg: &Pg) -> PgStatement {
        pg.build(self)
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

pub struct PgSelectBuilder<Columns> {
    pub(crate) plan: PgSelectPlan,
    pub(crate) columns: Columns,
    pub(crate) alias_count: usize,
}
