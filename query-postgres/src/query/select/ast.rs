use crate::query::{
    Expr,
    relation::{Relation, RelationId},
};

#[derive(Debug)]
pub(crate) struct SelectBody {
    pub(crate) from: Relation,
    pub(crate) joins: Vec<Join>,
    pub(crate) filters: Vec<Expr>,
    pub(crate) orders: Vec<Order>,
    pub(crate) limit: Option<u64>,
}

impl SelectBody {
    pub(crate) fn new(from: Relation) -> Self {
        Self {
            from,
            joins: Vec::new(),
            filters: Vec::new(),
            orders: Vec::new(),
            limit: None,
        }
    }

    pub(crate) fn freshen(
        &self,
        outer_bindings: &[(RelationId, RelationId)],
    ) -> (Self, Vec<(RelationId, RelationId)>) {
        let from = self.from.freshen(outer_bindings);
        let mut bindings = outer_bindings.to_vec();
        bindings.push((self.from.id, from.id));

        let joins = self.freshen_joins(outer_bindings, &mut bindings);
        let (filters, orders) = self.rebind_clauses(&bindings);

        let body = Self {
            from,
            joins,
            filters,
            orders,
            limit: self.limit,
        };
        (body, bindings)
    }

    fn freshen_joins(
        &self,
        outer_bindings: &[(RelationId, RelationId)],
        bindings: &mut Vec<(RelationId, RelationId)>,
    ) -> Vec<Join> {
        self.joins
            .iter()
            .map(|join| match join {
                Join::Inner { relation, on } => {
                    let replacement = relation.freshen(outer_bindings);
                    bindings.push((relation.id, replacement.id));
                    Join::Inner {
                        relation: replacement,
                        on: rebind(on.clone(), bindings),
                    }
                }
                Join::Lateral(relation) => {
                    let replacement = relation.freshen(bindings);
                    bindings.push((relation.id, replacement.id));
                    Join::Lateral(replacement)
                }
            })
            .collect()
    }

    fn rebind_clauses(&self, bindings: &[(RelationId, RelationId)]) -> (Vec<Expr>, Vec<Order>) {
        let filters = self
            .filters
            .iter()
            .map(|expr| rebind(expr.clone(), bindings))
            .collect();

        let orders = self
            .orders
            .iter()
            .map(|order| Order {
                expr: rebind(order.expr.clone(), bindings),
                descending: order.descending,
            })
            .collect();

        (filters, orders)
    }
}

#[derive(Debug)]
pub(crate) struct SelectAst {
    pub(crate) body: SelectBody,
    pub(crate) projection: Vec<Expr>,
}

impl SelectAst {
    pub(crate) fn freshen(
        &self,
        outer_bindings: &[(RelationId, RelationId)],
    ) -> (Self, Vec<(RelationId, RelationId)>) {
        let (body, bindings) = self.body.freshen(outer_bindings);
        let projection = self
            .projection
            .iter()
            .map(|expr| rebind(expr.clone(), &bindings))
            .collect();

        (Self { body, projection }, bindings)
    }
}

pub(crate) fn rebind<T>(expr: Expr<T>, bindings: &[(RelationId, RelationId)]) -> Expr<T> {
    expr.map_relations(&|relation_id| {
        bindings
            .iter()
            .rev()
            .find_map(|(original, replacement)| (*original == relation_id).then_some(*replacement))
            .unwrap_or(relation_id)
    })
}

#[derive(Debug)]
pub(crate) enum Join {
    Inner { relation: Relation, on: Expr },
    Lateral(Relation),
}

#[derive(Debug)]
pub(crate) struct Order {
    pub(crate) expr: Expr,
    pub(crate) descending: bool,
}
