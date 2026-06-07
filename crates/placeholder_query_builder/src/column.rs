use std::marker::PhantomData;

use crate::{
    expr::{BinaryOp, Column as ExprColumn, Expr, ExprFragment, Ident},
    value::Value,
};

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

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self {
            expr: self.expr.clone(),
            _value: PhantomData,
        }
    }
}
