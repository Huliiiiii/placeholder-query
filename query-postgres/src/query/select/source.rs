use crate::query::{
    Expr, Ident, Projection,
    columns::{Columns, Rebind},
    mode,
    relation::{Relation, UnnestColumn},
    table::TableSchema,
};

use super::{Select, ast::FromClause};

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

impl<P: Projection> Source<P> {
    pub(super) fn project(from: FromClause, projection: P) -> Self {
        let mut exprs = Vec::new();
        projection.visit_fields(&mut |expr: &Expr| exprs.push(expr.clone()));

        let relation = Relation::project(from, exprs);
        let projection = relation.bind_at(&projection, 0);
        Self {
            relation,
            projection,
        }
    }

    pub(super) fn with_maybe_tag(self) -> (Self, Expr<bool>) {
        let Source {
            relation,
            projection: (tag, projection),
        } = Source::project(
            FromClause::new(self.relation),
            (Expr::true_literal(), self.projection),
        );
        (
            Self {
                relation,
                projection,
            },
            tag,
        )
    }
}

impl<C: Columns<mode::Name>> From<TableSchema<C>> for Source<<C as Rebind>::Rebound<mode::Expr>> {
    fn from(table: TableSchema<C>) -> Self {
        let mut fields = Vec::new();
        table
            .columns
            .visit_fields(&mut |name: &Ident| fields.push(name.clone()));

        let relation = Relation::table(table.name, fields);
        let projection = relation.bind_at(&table.columns, 0);

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
        let mut exprs = Vec::new();
        projection.visit_fields(&mut |expr: &Expr| exprs.push(expr.clone()));

        let ast = super::ast::SelectAst {
            body,
            projection: exprs,
        };

        let relation = Relation::derived_table(Box::new(ast));
        let projection = relation.bind_at(&projection, 0);

        Self {
            relation,
            projection,
        }
    }
}
