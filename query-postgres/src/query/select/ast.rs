use crate::query::{
    Expr,
    relation::{Relation, RelationId},
};

#[derive(Clone, Copy, Debug)]
pub(crate) enum JoinKind {
    Cross,
    Inner,
    Left,
    Right,
    Full,
}

#[derive(Debug)]
pub(crate) struct Join {
    pub(crate) kind: JoinKind,
    pub(crate) lateral: bool,
    pub(crate) relation: Relation,
    pub(crate) on: Option<Expr>,
}

impl Join {
    fn freshen(&self, bindings: &mut Vec<(RelationId, RelationId)>) -> Self {
        let relation = self.relation.freshen(bindings);

        Self {
            kind: self.kind,
            lateral: self.lateral,
            relation,
            on: self.on.clone().map(|on| rebind(on, bindings)),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Order {
    pub(crate) expr: Expr,
    pub(crate) descending: bool,
}

#[derive(Debug)]
pub(crate) struct FromClause {
    pub(crate) source: Relation,
    pub(crate) joins: Vec<Join>,
}

impl FromClause {
    pub(crate) fn new(source: Relation) -> Self {
        Self {
            source,
            joins: Vec::new(),
        }
    }

    pub(crate) fn freshen(&self, bindings: &mut Vec<(RelationId, RelationId)>) -> Self {
        let source = self.source.freshen(bindings);
        let joins = self
            .joins
            .iter()
            .map(|join| join.freshen(bindings))
            .collect();
        Self { source, joins }
    }
}

#[derive(Debug)]
pub(crate) struct SelectBody {
    pub(crate) from: FromClause,
    pub(crate) filters: Vec<Expr>,
    pub(crate) orders: Vec<Order>,
    pub(crate) limit: Option<u64>,
}

impl SelectBody {
    pub(crate) fn new(from: Relation) -> Self {
        Self {
            from: FromClause::new(from),
            filters: Vec::new(),
            orders: Vec::new(),
            limit: None,
        }
    }

    pub(crate) fn freshen(
        &self,
        outer_bindings: &[(RelationId, RelationId)],
    ) -> (Self, Vec<(RelationId, RelationId)>) {
        let mut bindings = outer_bindings.to_vec();
        let from = self.from.freshen(&mut bindings);

        let (filters, orders) = self.rebind_clauses(&bindings);

        let body = Self {
            from,
            filters,
            orders,
            limit: self.limit,
        };
        (body, bindings)
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
    expr.map_columns(&mut |relation_id, field| {
        let relation_id = bindings
            .iter()
            .rev()
            .find_map(|(original, replacement)| (*original == relation_id).then_some(*replacement))
            .unwrap_or(relation_id);
        (relation_id, field)
    })
}
