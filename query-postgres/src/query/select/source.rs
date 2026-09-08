use std::sync::Arc;

use crate::query::{
    Ident, Projection,
    columns::{Columns, Rebind},
    mode,
    relation::{Relation, UnnestColumn},
    table::TableSchema,
};

use super::Select;

/// A typed relation occurrence used by `FROM` or `JOIN`.
pub struct Source<P> {
    pub(crate) relation: Relation,
    pub(crate) projection: P,
}

impl<P> Source<P> {
    #[doc(hidden)]
    pub fn unnest(columns: Vec<UnnestColumn>, bind: impl FnOnce(&Relation) -> P) -> Self {
        let relation = Relation::unnest(columns);
        let projection = bind(&relation);

        Self {
            relation,
            projection,
        }
    }
}

impl<C: Columns<mode::Name>> From<TableSchema<C>> for Source<<C as Rebind>::Rebound<mode::Expr>> {
    fn from(table: TableSchema<C>) -> Self {
        let mut names = Vec::new();
        table
            .columns
            .visit_fields(&mut |name: &Ident| names.push(name.clone()));

        let relation = Relation::table(table.name, names);
        let projection = relation.bind::<mode::Name, _>(&table.columns);

        Self {
            relation,
            projection,
        }
    }
}

impl<P: Projection> From<Select<P, ()>> for Source<P> {
    fn from(query: Select<P, ()>) -> Self {
        let Select {
            body, projection, ..
        } = query;
        let ast = super::ast::SelectAst {
            body,
            projection: projection.select_exprs(),
        };

        let relation = Relation::derived_table(Arc::new(ast));
        let projection = relation.bind::<mode::Expr, _>(&projection);

        Self {
            relation,
            projection,
        }
    }
}
