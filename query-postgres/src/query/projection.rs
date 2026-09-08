use super::{
    columns::{Columns, FieldTransformer, FieldVisitor, Rebind},
    expr::Expr,
    mode::{self, Mode},
};
use std::sync::Arc;

pub struct ColumnValue<T>(pub T);

pub trait Projection: Columns<mode::Expr, Rebound<mode::Expr> = Self> {
    type Fields;
    type Output;
    fn select_exprs(&self) -> Vec<Expr> {
        let mut exprs = Vec::new();
        self.visit_fields(&mut |expr: &Expr| exprs.push(expr.clone()));
        exprs
    }
    fn from_fields(&self, fields: Self::Fields) -> Self::Output;
}

#[derive_where::derive_where(Clone; P: Clone)]
pub struct MappedProjection<P, F> {
    projection: P,
    map: Arc<F>,
}

impl<P: Rebind, F> Rebind for MappedProjection<P, F> {
    type Rebound<M: Mode> = MappedProjection<<P as Rebind>::Rebound<M>, F>;
}

impl<M: Mode, P: Columns<M>, F> Columns<M> for MappedProjection<P, F> {
    fn visit_fields(&self, visitor: &mut impl FieldVisitor<M>) {
        self.projection.visit_fields(visitor);
    }

    fn map_fields<N: Mode>(
        &self,
        transformer: &mut impl FieldTransformer<M, N>,
    ) -> Self::Rebound<N> {
        MappedProjection {
            projection: self.projection.map_fields(transformer),
            map: self.map.clone(),
        }
    }
}

impl<P, F, T> Projection for MappedProjection<P, F>
where
    P: Projection,
    F: Fn(P::Output) -> T,
{
    type Fields = P::Fields;
    type Output = T;

    fn from_fields(&self, fields: Self::Fields) -> Self::Output {
        (self.map)(self.projection.from_fields(fields))
    }
}

pub trait ProjectionExt: Projection + Sized {
    fn map<T, F: Fn(Self::Output) -> T>(self, map: F) -> MappedProjection<Self, F> {
        MappedProjection {
            projection: self,
            map: Arc::new(map),
        }
    }
}

impl<P: Projection> ProjectionExt for P {}

impl<T> Projection for Expr<T> {
    type Fields = ColumnValue<T>;
    type Output = T;
    fn from_fields(&self, fields: Self::Fields) -> T {
        fields.0
    }
}

impl Projection for () {
    type Fields = ();
    type Output = ();
    fn from_fields(&self, _: ()) {}
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl Projection for #Tuple where #(#Member: Projection),* {
        type Fields = (#(#Member::Fields,)*);
        type Output = (#(#Member::Output,)*);
        fn from_fields(&self, fields: Self::Fields) -> Self::Output {
            (#(#self.from_fields(#fields),)*)
        }
    }
}
