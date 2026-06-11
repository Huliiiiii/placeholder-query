use std::marker::PhantomData;

use indexmap::IndexMap;
use placeholder_query_core::projection::Projection;
use placeholder_query_runtime::{FetchEnv, FetchKey};

use crate::query::select::{Pg, PgQueryCx, PgSelect, PgStatement};

type PgCollectFn<B, K> =
    dyn FnOnce(
        Vec<<B as PgBackend>::Row>,
    ) -> Result<IndexMap<K, <K as FetchKey>::Output>, <B as FetchEnv>::Error>;

pub trait PgBackend: FetchEnv {
    type Row;
}

pub trait PgFetchKey<B>: FetchKey
where
    B: PgBackend,
{
    fn batch(keys: &[Self]) -> impl Into<PgFetchBatch<B, Self>>;
}

pub struct PgFetchBatch<B, K>
where
    B: PgBackend,
    K: FetchKey,
{
    statement: PgStatement,
    collect: Box<PgCollectFn<B, K>>,
}

pub struct PgFetchBatchBuilder<K> {
    keys: Vec<K>,
}

pub struct PgBatchSelectBuilder<K, P> {
    keys: Vec<K>,
    select: PgSelect<P>,
}

pub struct PgKeyedBatch<K, V, F, O> {
    keys: Vec<K>,
    statement: PgStatement,
    key: F,
    _marker: PhantomData<fn() -> (V, O)>,
}

impl Pg {
    pub fn batch<K>(&self, keys: &[K]) -> PgFetchBatchBuilder<K>
    where
        K: Clone,
    {
        PgFetchBatchBuilder {
            keys: keys.to_vec(),
        }
    }
}

impl<B, K> PgFetchBatch<B, K>
where
    B: PgBackend,
    K: FetchKey,
{
    pub fn statement(&self) -> &PgStatement {
        &self.statement
    }

    pub fn collect(self, rows: Vec<B::Row>) -> Result<IndexMap<K, K::Output>, B::Error> {
        (self.collect)(rows)
    }
}

impl<K> PgFetchBatchBuilder<K> {
    pub fn select<P, Q>(
        self,
        build: impl FnOnce(PgQueryCx, &[K]) -> Q,
    ) -> PgBatchSelectBuilder<K, P>
    where
        Q: Into<PgSelect<P>>,
    {
        let select = Pg.select(|q| build(q, &self.keys));

        PgBatchSelectBuilder {
            keys: self.keys,
            select,
        }
    }
}

impl<K, P, V> PgBatchSelectBuilder<K, P>
where
    K: FetchKey,
    P: Projection<Output = V>,
{
    pub fn keyed_by(
        self,
        key: impl Fn(&V) -> K + 'static,
    ) -> PgKeyedBatch<K, V, impl Fn(&V) -> K + 'static, K::Output> {
        PgKeyedBatch {
            keys: self.keys,
            statement: self.select.build(),
            key,
            _marker: PhantomData,
        }
    }
}

impl<K, V, F, O> PgKeyedBatch<K, V, F, O>
where
    K: FetchKey,
    F: Fn(&V) -> K + 'static,
{
    pub fn collect<B>(
        self,
        collect: impl Fn(&K, Vec<V>) -> K::Output + 'static,
    ) -> PgFetchBatch<B, K>
    where
        B: PgBackend,
        V: TryFrom<B::Row, Error = B::Error> + 'static,
    {
        let keys = self.keys;
        let statement = self.statement;
        let key = self.key;

        PgFetchBatch {
            statement,
            collect: Box::new(move |rows| {
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
            }),
        }
    }
}

impl<B, K, V, F> From<PgKeyedBatch<K, V, F, Option<V>>> for PgFetchBatch<B, K>
where
    B: PgBackend,
    K: FetchKey<Output = Option<V>>,
    V: TryFrom<B::Row, Error = B::Error> + 'static,
    F: Fn(&V) -> K + 'static,
{
    fn from(fetch: PgKeyedBatch<K, V, F, Option<V>>) -> Self {
        fetch.collect(|_, rows| rows.into_iter().next())
    }
}

impl<B, K, V, F> From<PgKeyedBatch<K, V, F, Vec<V>>> for PgFetchBatch<B, K>
where
    B: PgBackend,
    K: FetchKey<Output = Vec<V>>,
    V: TryFrom<B::Row, Error = B::Error> + 'static,
    F: Fn(&V) -> K + 'static,
{
    fn from(fetch: PgKeyedBatch<K, V, F, Vec<V>>) -> Self {
        fetch.collect(|_, rows| rows)
    }
}
