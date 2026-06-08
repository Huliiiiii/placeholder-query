use std::marker::PhantomData;

use indexmap::IndexMap;
use placeholder_query_core::projection::Projection;
use placeholder_query_runtime::{FetchBackend, FetchBatch, FetchKey};

use crate::{
    PgFetchBackend,
    query::select::{Pg, PgQueryCx, PgSelect, PgStatement},
};

pub struct PgFetchBatchBuilder<B, K>
where
    B: FetchBackend<Request = PgStatement>,
{
    keys: Vec<K>,
    _backend: PhantomData<fn() -> B>,
}

pub struct PgBatchSelectBuilder<B, K, P>
where
    B: FetchBackend<Request = PgStatement>,
{
    keys: Vec<K>,
    select: PgSelect<P>,
    _backend: PhantomData<fn() -> B>,
}

pub struct PgKeyedBatch<B, K, V, F, O>
where
    B: FetchBackend<Request = PgStatement>,
{
    keys: Vec<K>,
    statement: PgStatement,
    key: F,
    _marker: PhantomData<fn() -> (B, V, O)>,
}

impl<Row, Error> PgFetchBackend<Row, Error> {
    pub fn batch<K>(keys: &[K]) -> PgFetchBatchBuilder<Self, K>
    where
        Self: FetchBackend<Request = PgStatement>,
        K: Clone,
    {
        PgFetchBatchBuilder {
            keys: keys.to_vec(),
            _backend: PhantomData,
        }
    }
}

impl<B, K> PgFetchBatchBuilder<B, K>
where
    B: FetchBackend<Request = PgStatement>,
{
    pub fn select<P, Q>(
        self,
        build: impl FnOnce(PgQueryCx, &[K]) -> Q,
    ) -> PgBatchSelectBuilder<B, K, P>
    where
        Q: Into<PgSelect<P>>,
    {
        let select = Pg.select(|q| build(q, &self.keys));

        PgBatchSelectBuilder {
            keys: self.keys,
            select,
            _backend: PhantomData,
        }
    }
}

impl<B, K, P, V> PgBatchSelectBuilder<B, K, P>
where
    B: FetchBackend<Request = PgStatement>,
    P: Projection<Output = V>,
{
    pub fn keyed_by(
        self,
        key: impl Fn(&V) -> K + 'static,
    ) -> PgKeyedBatch<B, K, V, impl Fn(&V) -> K + 'static, K::Output>
    where
        K: FetchKey<B>,
    {
        PgKeyedBatch {
            keys: self.keys,
            statement: self.select.build(),
            key,
            _marker: PhantomData,
        }
    }
}

impl<B, K, V, F, O> PgKeyedBatch<B, K, V, F, O>
where
    B: FetchBackend<Request = PgStatement>,
    K: FetchKey<B>,
    V: TryFrom<B::Row, Error = B::Error> + 'static,
    F: Fn(&V) -> K + 'static,
{
    pub fn collect(self, collect: impl Fn(&K, Vec<V>) -> K::Output + 'static) -> FetchBatch<B, K> {
        let keys = self.keys;
        let statement = self.statement;
        let key = self.key;

        FetchBatch::new(statement, move |rows| {
            let mut values = keys
                .iter()
                .cloned()
                .map(|key| (key, Vec::new()))
                .collect::<IndexMap<_, _>>();

            for row in rows {
                let value = V::try_from(row)?;
                let key = key(&value);
                if let Some(values) = values.get_mut(&key) {
                    values.push(value);
                }
            }

            Ok(keys
                .into_iter()
                .map(|key| {
                    let rows = values
                        .shift_remove(&key)
                        .expect("fetch key should have initialized row storage");
                    let output = collect(&key, rows);

                    (key, output)
                })
                .collect())
        })
    }
}

impl<B, K, V, F> From<PgKeyedBatch<B, K, V, F, Option<V>>> for FetchBatch<B, K>
where
    B: FetchBackend<Request = PgStatement>,
    K: FetchKey<B, Output = Option<V>>,
    V: TryFrom<B::Row, Error = B::Error> + 'static,
    F: Fn(&V) -> K + 'static,
{
    fn from(fetch: PgKeyedBatch<B, K, V, F, Option<V>>) -> Self {
        fetch.collect(|_, rows| rows.into_iter().next())
    }
}

impl<B, K, V, F> From<PgKeyedBatch<B, K, V, F, Vec<V>>> for FetchBatch<B, K>
where
    B: FetchBackend<Request = PgStatement>,
    K: FetchKey<B, Output = Vec<V>>,
    V: TryFrom<B::Row, Error = B::Error> + 'static,
    F: Fn(&V) -> K + 'static,
{
    fn from(fetch: PgKeyedBatch<B, K, V, F, Vec<V>>) -> Self {
        fetch.collect(|_, rows| rows)
    }
}
