use super::{
    Expr, Ident,
    mode::{self, Mode, Name},
};

pub trait Rebind {
    type Rebound<M: Mode>;
}

pub trait Columns<M: Mode>: Rebind {
    fn visit_fields(&self, visitor: &mut impl FieldVisitor<M>);

    fn map_fields<N: Mode>(
        &self,
        transformer: &mut impl FieldTransformer<M, N>,
    ) -> Self::Rebound<N>;
}

pub trait FieldVisitor<M: Mode> {
    fn visit<T>(&mut self, field: &M::Field<T>);
}

impl<F: FnMut(&Ident)> FieldVisitor<mode::Name> for F {
    fn visit<T>(&mut self, field: &Name<T>) {
        self(&field.0);
    }
}

impl<F: FnMut(&Expr)> FieldVisitor<mode::Expr> for F {
    fn visit<T>(&mut self, field: &Expr<T>) {
        self(&field.clone().erase());
    }
}

pub trait FieldTransformer<M: Mode, N: Mode> {
    fn transform<T>(&mut self, field: &M::Field<T>) -> N::Field<T>;
}

impl<T> Rebind for Name<T> {
    type Rebound<M: Mode> = M::Field<T>;
}

impl<T> Columns<mode::Name> for Name<T> {
    fn visit_fields(&self, visitor: &mut impl FieldVisitor<mode::Name>) {
        visitor.visit::<T>(self);
    }

    fn map_fields<N: Mode>(
        &self,
        transformer: &mut impl FieldTransformer<mode::Name, N>,
    ) -> N::Field<T> {
        transformer.transform::<T>(self)
    }
}

impl<T> Rebind for Expr<T> {
    type Rebound<M: Mode> = M::Field<T>;
}

impl<T> Columns<mode::Expr> for Expr<T> {
    fn visit_fields(&self, visitor: &mut impl FieldVisitor<mode::Expr>) {
        visitor.visit::<T>(self);
    }

    fn map_fields<N: Mode>(
        &self,
        transformer: &mut impl FieldTransformer<mode::Expr, N>,
    ) -> N::Field<T> {
        transformer.transform::<T>(self)
    }
}

impl Rebind for () {
    type Rebound<M: Mode> = ();
}

impl<M: Mode> Columns<M> for () {
    fn visit_fields(&self, _: &mut impl FieldVisitor<M>) {}

    fn map_fields<N: Mode>(&self, _: &mut impl FieldTransformer<M, N>) {}
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl Rebind for #Tuple where #(#Member: Rebind),* {
        type Rebound<N: Mode> = (#(#Member::Rebound<N>,)*);
    }
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl<M: Mode> Columns<M> for #Tuple where #(#Member: Columns<M>),* {
        fn visit_fields(&self, visitor: &mut impl FieldVisitor<M>) {
            #(#self.visit_fields(visitor);)*
        }

        fn map_fields<N: Mode>(&self, transformer: &mut impl FieldTransformer<M, N>) -> Self::Rebound<N> {
            (#(#self.map_fields(transformer),)*)
        }
    }
}
