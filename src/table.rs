use std::marker::PhantomData;

use crate::{
    expr::{BinaryOp, Column as ExprColumn, Expr, ExprFragment, Ident},
    value::Value,
};

pub trait Table {
    type Row;
    type Ref: Clone + Projection<Output = Self::Row>;

    const NAME: &'static str;

    fn bind(alias: Ident) -> Self::Ref;
}

#[derive(Debug)]
pub struct Column<T> {
    expr: ExprColumn,
    _value: PhantomData<fn() -> T>,
}

impl<T> Column<T> {
    pub fn new(table: impl Into<Ident>, name: impl Into<Ident>) -> Self {
        Self {
            expr: ExprColumn {
                schema: None,
                table: table.into(),
                name: name.into(),
            },
            _value: PhantomData,
        }
    }

    pub fn eq(self, right: impl Into<ExprFragment>) -> ExprFragment {
        self.binary(BinaryOp::Eq, right)
    }

    pub fn in_(self, values: impl IntoIterator<Item = impl Into<Value>>) -> ExprFragment {
        self.binary(BinaryOp::In, ExprFragment::values(values))
    }

    fn binary(self, op: BinaryOp, right: impl Into<ExprFragment>) -> ExprFragment {
        ExprFragment::binary(op, self.into(), right)
    }
}

impl<T> From<Column<T>> for Expr {
    fn from(value: Column<T>) -> Self {
        let Column { expr, .. } = value;

        Expr::Column(expr)
    }
}

impl Column<String> {
    pub fn like(self, pattern: impl Into<String>) -> ExprFragment {
        let pattern: String = pattern.into();
        self.binary(BinaryOp::Like, pattern)
    }
}

pub trait Projection {
    type Row;
    type Output;

    fn columns(&self) -> Vec<ExprFragment>;

    fn bind(self, row: Self::Row) -> Self::Output;
}

pub struct MappedProjection<P, F> {
    projection: P,
    bind: F,
}

impl<P, F, T> Projection for MappedProjection<P, F>
where
    P: Projection,
    F: FnOnce(P::Output) -> T,
{
    type Row = P::Row;
    type Output = T;

    fn columns(&self) -> Vec<ExprFragment> {
        self.projection.columns()
    }

    fn bind(self, row: Self::Row) -> Self::Output {
        (self.bind)(self.projection.bind(row))
    }
}

pub trait ProjectionExt: Projection + Sized {
    fn map<T>(
        self,
        bind: impl FnOnce(Self::Output) -> T,
    ) -> MappedProjection<Self, impl FnOnce(Self::Output) -> T> {
        MappedProjection {
            projection: self,
            bind,
        }
    }
}

impl<P: Projection> ProjectionExt for P {}

impl<T> Projection for Column<T> {
    type Row = T;
    type Output = T;

    fn columns(&self) -> Vec<ExprFragment> {
        vec![self.clone().into()]
    }

    fn bind(self, row: Self::Row) -> Self::Output {
        row
    }
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            expr: self.expr.clone(),
            _value: PhantomData,
        }
    }
}

impl<A, B> Projection for (Column<A>, Column<B>) {
    type Row = (A, B);
    type Output = (A, B);

    fn columns(&self) -> Vec<ExprFragment> {
        vec![self.0.clone().into(), self.1.clone().into()]
    }

    fn bind(self, row: Self::Row) -> Self::Output {
        row
    }
}

impl<A, B, C> Projection for (Column<A>, Column<B>, Column<C>) {
    type Row = (A, B, C);
    type Output = (A, B, C);

    fn columns(&self) -> Vec<ExprFragment> {
        vec![
            self.0.clone().into(),
            self.1.clone().into(),
            self.2.clone().into(),
        ]
    }

    fn bind(self, row: Self::Row) -> Self::Output {
        row
    }
}

impl<A, B, C, D> Projection for (Column<A>, Column<B>, Column<C>, Column<D>) {
    type Row = (A, B, C, D);
    type Output = (A, B, C, D);

    fn columns(&self) -> Vec<ExprFragment> {
        vec![
            self.0.clone().into(),
            self.1.clone().into(),
            self.2.clone().into(),
            self.3.clone().into(),
        ]
    }

    fn bind(self, row: Self::Row) -> Self::Output {
        row
    }
}
