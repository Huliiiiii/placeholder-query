use placeholder_query_core::ident::Ident;

use crate::{
    backend::Pg,
    query::{Expr, PgQueryCx, Projection, Table},
};

use super::{
    plan::{PgJoin, PgSelect, PgSelectBuilder, PgSelectPlan, PgTableRef},
    predicate::IntoPredicates,
};

fn table_alias(index: usize) -> Ident {
    format!("t{index}").into()
}

impl Pg {
    pub fn select<P, Q>(&self, build: impl FnOnce(PgQueryCx) -> Q) -> PgSelect<P>
    where
        Q: Into<PgSelect<P>>,
    {
        build(PgQueryCx).into()
    }

    pub fn from<T: Table>(self, _table: T) -> PgSelectBuilder<T::Columns> {
        PgQueryCx.from(_table)
    }
}

impl PgQueryCx {
    pub fn from<T: Table>(self, _table: T) -> PgSelectBuilder<T::Columns> {
        let alias = table_alias(0);
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
        let alias = table_alias(self.alias_count);
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
        let on = self.plan.exprs.append(on(columns.clone()).into());

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
        P: IntoPredicates,
    {
        self.plan.filters.extend(
            filter(self.columns.clone())
                .into_predicates()
                .map(|expr| self.plan.exprs.append(expr.into())),
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
            .map(|expr| self.plan.exprs.append(expr.into()))
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
            .map(|expr| value.plan.exprs.append(expr.into()))
            .collect();

        PgSelect {
            plan: value.plan,
            _projection: value.columns,
        }
    }
}
