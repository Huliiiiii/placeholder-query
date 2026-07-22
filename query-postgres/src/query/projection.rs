use super::{column::Column, expr::Expr};

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

fortuples::fortuples! {
    #[tuples::min_size(2)]
    #[tuples::max_size(21)]
    impl Projection for (#(Column<#Member>),*) {
        type Fields = (#(#Member),*);
        type Output = (#(#Member),*);

        fn select_exprs(&self) -> Vec<Expr> {
            vec![#(#self.clone().into()),*]
        }

        fn from_fields(&self, fields: Self::Fields) -> Self::Output {
            fields
        }
    }
}
