use placeholder_query_core::{
    expr::{Expr, Ident},
    projection::Projection,
    table::Table,
};

use super::{
    builder::Pg,
    plan::{PgJoin, PgSelect, PgSelectBuilder, PgSelectPlan, PgTableRef},
};

#[derive(Clone, Copy, Debug)]
pub struct PgQueryCx;

impl Pg {
    pub fn query<Q>(&self, build: impl FnOnce(PgQueryCx) -> Q) -> Q {
        build(PgQueryCx)
    }

    pub fn select<P, Q>(&self, build: impl FnOnce(PgQueryCx) -> Q) -> PgSelect<P>
    where
        Q: Into<PgSelect<P>>,
    {
        self.query(build).into()
    }

    pub fn from<T: Table>(self, _table: T) -> PgSelectBuilder<T::Columns> {
        PgQueryCx.from(_table)
    }
}

impl PgQueryCx {
    pub fn from<T: Table>(self, _table: T) -> PgSelectBuilder<T::Columns> {
        let alias: Ident = "t0".into();
        let columns = T::bind_alias(alias.clone());

        PgSelectBuilder {
            plan: PgSelectPlan::new(PgTableRef {
                name: T::NAME.into(),
                alias,
            }),
            columns,
            alias_count: 1,
        }
    }
}

impl<Columns> PgSelectBuilder<Columns> {
    fn next_alias(&mut self) -> Ident {
        let alias = format!("t{}", self.alias_count).into();
        self.alias_count += 1;
        alias
    }

    pub fn join<T: Table>(
        mut self,
        _table: T,
        on: impl FnOnce((Columns, T::Columns)) -> Expr,
    ) -> PgSelectBuilder<(Columns, T::Columns)>
    where
        Columns: Clone,
    {
        let alias = self.next_alias();
        let right = T::bind_alias(alias.clone());
        let columns = (self.columns, right.clone());
        let on = self.plan.exprs.append(on(columns.clone()));

        self.plan.joins.push(PgJoin {
            table: PgTableRef {
                name: T::NAME.into(),
                alias,
            },
            on,
        });

        PgSelectBuilder {
            plan: self.plan,
            columns,
            alias_count: self.alias_count,
        }
    }

    pub fn filter<P>(mut self, filter: impl FnOnce(Columns) -> P) -> Self
    where
        Columns: Clone,
        P: IntoIterator<Item = Expr>,
    {
        self.plan.filters.extend(
            filter(self.columns.clone())
                .into_iter()
                .map(|expr| self.plan.exprs.append(expr)),
        );

        self
    }

    pub fn project<P>(mut self, project: impl FnOnce(Columns) -> P) -> PgSelect<P>
    where
        P: Projection,
    {
        let projection = project(self.columns);
        self.plan.select = projection
            .select_exprs()
            .into_iter()
            .map(|expr| self.plan.exprs.append(expr))
            .collect();

        PgSelect {
            plan: self.plan,
            _projection: projection,
        }
    }
}

impl<Columns> From<PgSelectBuilder<Columns>> for PgSelect<Columns>
where
    Columns: Projection,
{
    fn from(mut value: PgSelectBuilder<Columns>) -> Self {
        value.plan.select = value
            .columns
            .select_exprs()
            .into_iter()
            .map(|expr| value.plan.exprs.append(expr))
            .collect();

        PgSelect {
            plan: value.plan,
            _projection: value.columns,
        }
    }
}
