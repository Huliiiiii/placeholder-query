use crate::query::Expr;

pub trait IntoPredicates {
    fn into_predicates(self) -> impl Iterator<Item = Expr<bool>>;
}

impl IntoPredicates for Expr<bool> {
    fn into_predicates(self) -> impl Iterator<Item = Expr<bool>> {
        std::iter::once(self)
    }
}

impl<I> IntoPredicates for I
where
    I: IntoIterator<Item = Expr<bool>>,
{
    fn into_predicates(self) -> impl Iterator<Item = Expr<bool>> {
        self.into_iter()
    }
}
