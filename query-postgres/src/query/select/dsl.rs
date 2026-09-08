use std::marker::PhantomData;

use crate::query::{
    Expr, OrderExpr,
    columns::{Columns, FieldTransformer},
    mode,
    relation::RelationId,
};

use super::{
    Select,
    ast::{Join, Order, rebind},
    predicate::IntoPredicates,
    source::Source,
};

impl<Columns, Input: ?Sized> Select<Columns, Input> {
    pub fn join<P>(
        mut self,
        source: impl Into<Source<P>>,
        on: impl FnOnce(&(Columns, P)) -> Expr<bool>,
    ) -> Select<(Columns, P), Input> {
        let Source {
            relation,
            projection,
        } = source.into();
        let columns = (self.projection, projection);

        self.body.joins.push(Join::Inner {
            relation,
            on: on(&columns).erase(),
        });

        Select {
            body: self.body,
            projection: columns,
            input_scope: self.input_scope,
            _input: PhantomData,
        }
    }

    pub fn filter<P: IntoPredicates>(mut self, filter: impl FnOnce(&Columns) -> P) -> Self {
        self.body
            .filters
            .extend(filter(&self.projection).into_predicates().map(Expr::erase));
        self
    }

    pub fn order_by(mut self, order: impl FnOnce(&Columns) -> OrderExpr) -> Self {
        let order = order(&self.projection);
        self.body.orders.push(Order {
            expr: order.expr,
            descending: order.descending,
        });
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.body.limit = Some(limit);
        self
    }

    pub fn lateral<P, Q: Into<Source<P>>>(
        mut self,
        build: impl FnOnce(Columns) -> Q,
    ) -> Select<P, Input> {
        let source = build(self.projection).into();
        self.body.joins.push(Join::Lateral(source.relation));
        Select {
            body: self.body,
            projection: source.projection,
            input_scope: self.input_scope,
            _input: PhantomData,
        }
    }

    pub fn map<P>(self, map: impl FnOnce(Columns) -> P) -> Select<P, Input> {
        Select {
            body: self.body,
            projection: map(self.projection),
            input_scope: self.input_scope,
            _input: PhantomData,
        }
    }
}

impl<P, Input: ?Sized> Clone for Select<P, Input>
where
    P: Columns<mode::Expr, Rebound<mode::Expr> = P>,
{
    fn clone(&self) -> Self {
        struct Rebind<'a> {
            bindings: &'a [(RelationId, RelationId)],
        }

        impl FieldTransformer<mode::Expr, mode::Expr> for Rebind<'_> {
            fn transform<T>(&mut self, field: &Expr<T>) -> Expr<T> {
                rebind(field.clone(), self.bindings)
            }
        }

        let (body, bindings) = self.body.freshen(&[]);
        let projection = self.projection.map_fields(&mut Rebind {
            bindings: &bindings,
        });
        Self {
            body,
            projection,
            input_scope: self.input_scope,
            _input: PhantomData,
        }
    }
}
