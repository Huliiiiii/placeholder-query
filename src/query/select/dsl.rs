use std::marker::PhantomData;

use crate::{
    expr::{ExprFragment, Ident},
    table::{Projection, Table},
};

use super::{
    builder::PgQueryBuilder,
    plan::{PgFrom, PgJoin, PgQueryPlan, PgSelect, PgSelectQuery, PgTableRef},
};

#[derive(Clone, Copy, Debug, Default)]
pub struct PgQueryCx;

impl PgQueryBuilder {
    pub fn query<Q>(&self, build: impl FnOnce(PgQueryCx) -> Q) -> Q {
        build(PgQueryCx)
    }

    pub fn select<Q>(&self, build: impl FnOnce(PgQueryCx) -> Q) -> PgSelect<'_>
    where
        Q: Into<PgQueryPlan>,
    {
        PgSelect {
            builder: self,
            plan: self.query(build).into(),
        }
    }

    pub fn from<T: Table>(self, _table: T) -> PgFrom<T::Ref> {
        PgQueryCx.from(_table)
    }
}

impl PgQueryCx {
    pub fn from<T: Table>(self, _table: T) -> PgFrom<T::Ref> {
        let alias: Ident = "t0".into();
        let refs = T::bind(alias.clone());

        PgFrom {
            plan: PgQueryPlan::new(PgTableRef {
                name: T::NAME.into(),
                alias,
            }),
            refs,
            alias_count: 1,
        }
    }
}

impl<Refs: Clone> PgFrom<Refs> {
    fn next_alias(&mut self) -> Ident {
        let alias = format!("t{}", self.alias_count).into();
        self.alias_count += 1;
        alias
    }

    pub fn join<T: Table>(
        mut self,
        _table: T,
        on: impl FnOnce((Refs, T::Ref)) -> ExprFragment,
    ) -> PgFrom<(Refs, T::Ref)> {
        let alias = self.next_alias();
        let right = T::bind(alias.clone());
        let refs = (self.refs, right.clone());
        let on = self.plan.exprs.append(on(refs.clone()));

        self.plan.joins.push(PgJoin {
            table: PgTableRef {
                name: T::NAME.into(),
                alias,
            },
            on,
        });

        PgFrom {
            plan: self.plan,
            refs,
            alias_count: self.alias_count,
        }
    }

    pub fn filter<P>(mut self, filter: impl FnOnce(Refs) -> P) -> Self
    where
        P: IntoIterator<Item = ExprFragment>,
    {
        self.plan.filters.extend(
            filter(self.refs.clone())
                .into_iter()
                .map(|expr| self.plan.exprs.append(expr)),
        );

        self
    }

    pub fn project<P: Projection>(
        mut self,
        project: impl FnOnce(Refs) -> P,
    ) -> PgSelectQuery<P::Output, P> {
        let projection = project(self.refs.clone());
        self.plan.select = projection
            .columns()
            .into_iter()
            .map(|expr| self.plan.exprs.append(expr))
            .collect();

        PgSelectQuery {
            plan: self.plan,
            _output: PhantomData,
        }
    }
}

impl<Refs> From<PgFrom<Refs>> for PgQueryPlan
where
    Refs: Clone + Projection,
{
    fn from(mut value: PgFrom<Refs>) -> Self {
        value.plan.select = value
            .refs
            .columns()
            .into_iter()
            .map(|expr| value.plan.exprs.append(expr))
            .collect();

        value.plan
    }
}

impl<T, P> From<PgSelectQuery<T, P>> for PgQueryPlan {
    fn from(value: PgSelectQuery<T, P>) -> Self {
        value.plan
    }
}
