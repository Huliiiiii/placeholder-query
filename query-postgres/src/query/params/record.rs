use std::ops::Deref;

use crate::query::{
    columns::{Columns, FieldTransformer, Rebind},
    mode::{self, Name},
    relation::UnnestColumn,
    select::source::Source,
};

use super::{
    ArrayElement, InputScope, Param, QueryInput,
    mode::{Array, Scalar},
};

pub trait InputRecord {
    type Fields: RecordFields;
    type Names: Columns<mode::Name>;

    const NAMES: &'static [&'static str];

    fn names() -> Self::Names;
    fn fields(&self) -> <Self::Fields as RecordFields>::Values<'_>;
}

impl<K: InputRecord> QueryInput for K {
    type Params = <K::Names as Rebind>::Rebound<Scalar>;
    type Values<'a>
        = <K::Fields as RecordFields>::Values<'a>
    where
        K: 'a;

    fn params(scope: InputScope) -> Self::Params {
        K::names().map_fields(&mut BindParams::new(scope))
    }

    fn values(&self) -> Self::Values<'_> {
        self.fields()
    }
}

impl<K: InputRecord> QueryInput for [K]
where
    K::Fields: ArrayFields,
{
    type Params = <K::Fields as ArrayFields>::Params<K>;
    type Values<'a>
        = <K::Fields as ArrayFields>::Arrays<'a>
    where
        K: 'a;

    fn params(scope: InputScope) -> Self::Params {
        K::Fields::params::<K>(scope)
    }

    fn values(&self) -> Self::Values<'_> {
        K::Fields::transpose(self.iter().map(K::fields))
    }
}

struct BindParams {
    scope: InputScope,
    index: u16,
}

impl BindParams {
    fn new(scope: InputScope) -> Self {
        Self { scope, index: 0 }
    }

    fn next<T>(&mut self) -> Param<T> {
        let param = self.scope.param(self.index);

        self.index += 1;

        param
    }
}

impl FieldTransformer<mode::Name, Scalar> for BindParams {
    fn transform<T>(&mut self, _: &Name<T>) -> Param<T> {
        self.next()
    }
}

impl FieldTransformer<mode::Name, Array> for BindParams {
    fn transform<T>(&mut self, _: &Name<T>) -> Param<Vec<T>> {
        self.next()
    }
}

pub struct ArrayParams<K: InputRecord> {
    fields: <K::Names as Rebind>::Rebound<Array>,
    scope: InputScope,
}

impl<K: InputRecord> Deref for ArrayParams<K> {
    type Target = <K::Names as Rebind>::Rebound<Array>;

    fn deref(&self) -> &Self::Target {
        &self.fields
    }
}

impl<K: InputRecord> ArrayParams<K>
where
    K::Fields: ArrayFields,
{
    pub fn unnest(self) -> Source<<K::Names as Rebind>::Rebound<mode::Expr>> {
        let names = K::names();

        Source::unnest(K::Fields::columns(self.scope, K::NAMES), |relation| {
            relation.bind::<mode::Name, _>(&names)
        })
    }
}

pub trait RecordFields {
    type Values<'a>;
}

impl RecordFields for () {
    type Values<'a> = ();
}

pub trait ArrayFields: RecordFields {
    type Params<K: InputRecord<Fields = Self>>;
    type Arrays<'a>;

    fn params<K: InputRecord<Fields = Self>>(scope: InputScope) -> Self::Params<K>;
    fn transpose<'a>(rows: impl Iterator<Item = Self::Values<'a>>) -> Self::Arrays<'a>;
    fn columns(scope: InputScope, names: &[&'static str]) -> Vec<UnnestColumn>;
}

impl ArrayFields for () {
    type Params<K: InputRecord<Fields = Self>> = ();
    type Arrays<'a> = ();

    fn params<K: InputRecord<Fields = Self>>(_: InputScope) -> Self::Params<K> {}

    fn transpose<'a>(rows: impl Iterator<Item = Self::Values<'a>>) -> Self::Arrays<'a> {
        for () in rows {}
    }

    fn columns(_: InputScope, _: &[&'static str]) -> Vec<UnnestColumn> {
        Vec::new()
    }
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl RecordFields for #Tuple where #(#Member: 'static),* {
        type Values<'a> = (#(&'a #Member,)*);
    }
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl ArrayFields for #Tuple where #(#Member: ArrayElement + 'static),* {
        type Params<K: InputRecord<Fields = Self>> = ArrayParams<K>;
        type Arrays<'a> = (#(Vec<&'a #Member>,)*);

        fn params<K: InputRecord<Fields = Self>>(scope: InputScope) -> Self::Params<K> {
            ArrayParams {
                fields: K::names().map_fields(&mut BindParams::new(scope)),
                scope,
            }
        }

        fn transpose<'a>(rows: impl Iterator<Item = Self::Values<'a>>) -> Self::Arrays<'a> {
            let mut arrays = (#(Vec::<&'a #Member>::new(),)*);

            for row in rows {
                #(#arrays.push(#row);)*
            }

            arrays
        }

        fn columns(scope: InputScope, names: &[&'static str]) -> Vec<UnnestColumn> {
            let mut names = names.iter().enumerate();

            vec![#({
                let (index, &name) = names.next().expect("record layout supplies every field name");
                UnnestColumn::new::<#Member>(name, scope.param(index as u16))
            },)*]
        }
    }
}
