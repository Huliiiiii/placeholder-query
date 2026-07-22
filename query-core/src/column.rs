use std::marker::PhantomData;

use crate::{
    backend::QueryBackend,
    expr::{ColumnRef, Expr, ExprNode},
    ident::Ident,
};

#[derive(Debug)]
pub struct Column<B: QueryBackend, T> {
    column_ref: ColumnRef,
    _value: PhantomData<fn() -> (B, T)>,
}

impl<B, T> Column<B, T>
where
    B: QueryBackend,
{
    pub fn new(table_alias: impl Into<Ident>, name: impl Into<Ident>) -> Self {
        Self {
            column_ref: ColumnRef {
                schema: None,
                table_alias: table_alias.into(),
                name: name.into(),
            },
            _value: PhantomData,
        }
    }
}

impl<B, T> From<Column<B, T>> for ExprNode<B>
where
    B: QueryBackend,
{
    fn from(value: Column<B, T>) -> Self {
        let Column { column_ref, .. } = value;

        Self::Column(column_ref)
    }
}

impl<B, T> From<Column<B, T>> for Expr<B>
where
    B: QueryBackend,
{
    fn from(value: Column<B, T>) -> Self {
        ExprNode::from(value).into()
    }
}

impl<B, T> Clone for Column<B, T>
where
    B: QueryBackend,
{
    fn clone(&self) -> Self {
        Self {
            column_ref: self.column_ref.clone(),
            _value: PhantomData,
        }
    }
}
