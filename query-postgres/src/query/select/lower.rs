use crate::query::{
    Expr,
    relation::{Relation, RelationId, RelationNode},
};

use super::ast::{FromClause, Join, Order, SelectAst, SelectBody};

type Column = (RelationId, u16);
type Bindings = Vec<(Column, Column)>;

pub(super) fn lower_select(query: SelectAst, bindings: &[(Column, Column)]) -> SelectAst {
    let (from, bindings) = lower_from(query.body.from, bindings.to_vec());
    let projection = query
        .projection
        .into_iter()
        .map(|expr| rebind(expr, &bindings))
        .collect();
    let filters = query
        .body
        .filters
        .into_iter()
        .map(|expr| rebind(expr, &bindings))
        .collect();
    let orders = query
        .body
        .orders
        .into_iter()
        .map(|order| Order {
            expr: rebind(order.expr, &bindings),
            ..order
        })
        .collect();

    SelectAst {
        body: SelectBody {
            from,
            filters,
            orders,
            ..query.body
        },
        projection,
    }
}

fn lower_from(from: FromClause, mut bindings: Bindings) -> (FromClause, Bindings) {
    let (source, replacements) = lower_relation(from.source, &bindings);
    bindings.extend(replacements);

    let mut joins = Vec::with_capacity(from.joins.len());
    for join in from.joins {
        let (relation, replacements) = lower_relation(join.relation, &bindings);
        bindings.extend(replacements);
        joins.push(Join {
            relation,
            on: join.on.map(|on| rebind(on, &bindings)),
            ..join
        });
    }

    (FromClause { source, joins }, bindings)
}

fn lower_relation(relation: Relation, bindings: &[(Column, Column)]) -> (Relation, Bindings) {
    let (node, replacements) = match relation.node {
        RelationNode::Projection { from, exprs } => {
            let (from, mut input_bindings) = lower_from(*from, bindings.to_vec());
            let mut exprs: Vec<_> = exprs
                .into_iter()
                .map(|expr| rebind(expr, &input_bindings))
                .collect();

            // Keep the typed projection prefix stable; hidden columns follow it.
            let mut exports = Vec::new();
            let rels =
                std::iter::once(&from.source).chain(from.joins.iter().map(|join| &join.relation));
            for rel in rels {
                let columns = match &rel.node {
                    RelationNode::Table { fields, .. } => fields.len(),
                    RelationNode::Unnest(columns) => columns.len(),
                    RelationNode::DerivedTable(query) => query.projection.len(),
                    RelationNode::Projection { .. } => unreachable!("projection should be lowered"),
                };
                for index in 0..columns {
                    let field =
                        u16::try_from(index).expect("relation exceeds 65536 output columns");
                    let column = rel.column(field);
                    let index = exprs
                        .iter()
                        .position(|expr| *expr == column)
                        .unwrap_or_else(|| {
                            let index = exprs.len();
                            exprs.push(column);
                            index
                        });
                    let output =
                        u16::try_from(index).expect("relation exceeds 65536 output columns");
                    exports.push(((rel.id, field), (relation.id, output)));
                }
            }

            let mut replacements = input_bindings.split_off(bindings.len());
            for (_, column) in &mut replacements {
                *column = rebind_column(*column, &exports);
            }
            replacements.extend(exports);

            let query = SelectAst {
                body: SelectBody {
                    from,
                    filters: Vec::new(),
                    orders: Vec::new(),
                    limit: None,
                },
                projection: exprs,
            };
            (RelationNode::DerivedTable(Box::new(query)), replacements)
        }
        RelationNode::DerivedTable(query) => (
            RelationNode::DerivedTable(Box::new(lower_select(*query, bindings))),
            Vec::new(),
        ),
        node => (node, Vec::new()),
    };

    (Relation { node, ..relation }, replacements)
}

fn rebind(expr: Expr, bindings: &[(Column, Column)]) -> Expr {
    expr.map_columns(&mut |relation, field| rebind_column((relation, field), bindings))
}

fn rebind_column(column: Column, bindings: &[(Column, Column)]) -> Column {
    bindings
        .iter()
        .rev()
        .find_map(|(original, replacement)| (*original == column).then_some(*replacement))
        .unwrap_or(column)
}
