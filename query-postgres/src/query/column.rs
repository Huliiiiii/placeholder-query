use placeholder_query_core::{column::Column as CoreColumn, expr::Expr as CoreExpr, ident::Ident};

use crate::{backend::Pg, value::Value};

use super::{expr::Expr, operator::BinaryOp};

#[derive(Debug)]
pub struct Column<T>(CoreColumn<Pg, T>);

impl<T> Column<T> {
    pub fn new(table_alias: impl Into<Ident>, name: impl Into<Ident>) -> Self {
        Self(CoreColumn::new(table_alias, name))
    }

    pub fn eq(self, right: impl Into<Expr>) -> Expr {
        let right = right.into();

        CoreExpr::binary(BinaryOp::Eq, self.into(), right).into()
    }

    pub fn in_(self, values: impl IntoIterator<Item = impl Into<Value>>) -> Expr {
        let right = CoreExpr::values(values);

        CoreExpr::binary(BinaryOp::In, self.into(), right).into()
    }
}

impl Column<String> {
    pub fn like(self, pattern: impl Into<String>) -> Expr {
        let right = CoreExpr::value(Value::from(pattern.into()));

        CoreExpr::binary(BinaryOp::Like, self.into(), right).into()
    }
}

impl<T> From<Column<T>> for CoreColumn<Pg, T> {
    fn from(value: Column<T>) -> Self {
        value.0
    }
}

impl<T> From<Column<T>> for CoreExpr<Pg> {
    fn from(value: Column<T>) -> Self {
        CoreColumn::from(value).into()
    }
}

impl<T> Clone for Column<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
