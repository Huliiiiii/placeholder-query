use crate::query::Expr;

pub trait IntoPredicates {
    fn into_predicates(self) -> impl Iterator<Item = Expr>;
}

impl IntoPredicates for Expr {
    fn into_predicates(self) -> impl Iterator<Item = Expr> {
        std::iter::once(self)
    }
}

impl<I> IntoPredicates for I
where
    I: IntoIterator<Item = Expr>,
{
    fn into_predicates(self) -> impl Iterator<Item = Expr> {
        self.into_iter()
    }
}
