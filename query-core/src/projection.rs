use crate::{column::Column, expr::Expr};

pub trait Projection {
    type Fields;
    type Output;

    fn select_exprs(&self) -> Vec<Expr>;

    fn from_fields(&self, fields: Self::Fields) -> Self::Output;
}

pub struct MappedProjection<P, F> {
    projection: P,
    map: F,
}

impl<P, F, T> Projection for MappedProjection<P, F>
where
    P: Projection,
    F: Fn(P::Output) -> T,
{
    type Fields = P::Fields;
    type Output = T;

    fn select_exprs(&self) -> Vec<Expr> {
        self.projection.select_exprs()
    }

    fn from_fields(&self, fields: Self::Fields) -> Self::Output {
        (self.map)(self.projection.from_fields(fields))
    }
}

pub trait ProjectionExt: Projection + Sized {
    fn map<T>(
        self,
        map: impl Fn(Self::Output) -> T,
    ) -> MappedProjection<Self, impl Fn(Self::Output) -> T> {
        MappedProjection {
            projection: self,
            map,
        }
    }
}

impl<P: Projection> ProjectionExt for P {}

impl<T> Projection for Column<T> {
    type Fields = T;
    type Output = T;

    fn select_exprs(&self) -> Vec<Expr> {
        vec![self.clone().into()]
    }

    fn from_fields(&self, fields: Self::Fields) -> Self::Output {
        fields
    }
}

impl<A, B> Projection for (Column<A>, Column<B>) {
    type Fields = (A, B);
    type Output = (A, B);

    fn select_exprs(&self) -> Vec<Expr> {
        vec![self.0.clone().into(), self.1.clone().into()]
    }

    fn from_fields(&self, fields: Self::Fields) -> Self::Output {
        fields
    }
}

impl<A, B, C> Projection for (Column<A>, Column<B>, Column<C>) {
    type Fields = (A, B, C);
    type Output = (A, B, C);

    fn select_exprs(&self) -> Vec<Expr> {
        vec![
            self.0.clone().into(),
            self.1.clone().into(),
            self.2.clone().into(),
        ]
    }

    fn from_fields(&self, fields: Self::Fields) -> Self::Output {
        fields
    }
}

impl<A, B, C, D> Projection for (Column<A>, Column<B>, Column<C>, Column<D>) {
    type Fields = (A, B, C, D);
    type Output = (A, B, C, D);

    fn select_exprs(&self) -> Vec<Expr> {
        vec![
            self.0.clone().into(),
            self.1.clone().into(),
            self.2.clone().into(),
            self.3.clone().into(),
        ]
    }

    fn from_fields(&self, fields: Self::Fields) -> Self::Output {
        fields
    }
}
