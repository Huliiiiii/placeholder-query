use std::marker::PhantomData;

use crate::query::{
    Expr, OrderExpr,
    columns::{Columns, FieldTransformer},
    mode,
    relation::RelationId,
};

use super::{Select, ast::rebind, predicate::IntoPredicates};

impl<Columns, Input: ?Sized> Select<Columns, Input> {
    pub fn filter<P: IntoPredicates>(mut self, filter: impl FnOnce(&Columns) -> P) -> Self {
        self.body
            .filters
            .extend(filter(&self.projection).into_predicates().map(Expr::erase));
        self
    }

    pub fn order_by(mut self, order: impl FnOnce(&Columns) -> OrderExpr) -> Self {
        self.body.order_by(order(&self.projection));
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.body.limit = Some(limit);
        self
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

        let (body, bindings) = self.body.freshen(Vec::new());
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
