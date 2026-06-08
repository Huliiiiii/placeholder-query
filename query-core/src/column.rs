use std::marker::PhantomData;

use crate::{
    expr::{BinaryOp, ColumnRef, Expr, ExprNode, Ident},
    value::Value,
};

#[derive(Debug)]
pub struct Column<T> {
    column_ref: ColumnRef,
    _value: PhantomData<fn() -> T>,
}

impl<T> Column<T> {
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

    pub fn eq(self, right: impl Into<Expr>) -> Expr {
        self.binary(BinaryOp::Eq, right)
    }

    pub fn in_(self, values: impl IntoIterator<Item = impl Into<Value>>) -> Expr {
        self.binary(BinaryOp::In, Expr::values(values))
    }

    fn binary(self, op: BinaryOp, right: impl Into<Expr>) -> Expr {
        Expr::binary(op, self.into(), right)
    }
}

impl<T> From<Column<T>> for ExprNode {
    fn from(value: Column<T>) -> Self {
        let Column { column_ref, .. } = value;

        Self::Column(column_ref)
    }
}

impl Column<String> {
    pub fn like(self, pattern: impl Into<String>) -> Expr {
        let pattern: String = pattern.into();
        self.binary(BinaryOp::Like, pattern)
    }
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            column_ref: self.column_ref.clone(),
            _value: PhantomData,
        }
    }
}
