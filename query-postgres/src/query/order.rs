use super::Expr;

pub struct OrderExpr {
    pub(crate) expr: Expr,
    pub(crate) descending: bool,
}

impl<T> Expr<T> {
    pub fn asc(self) -> OrderExpr {
        OrderExpr {
            expr: self.erase(),
            descending: false,
        }
    }

    pub fn desc(self) -> OrderExpr {
        OrderExpr {
            expr: self.erase(),
            descending: true,
        }
    }
}
