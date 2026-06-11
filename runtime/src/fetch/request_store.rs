use std::{
    any::{Any, TypeId},
    future::Future,
    pin::Pin,
};

use futures_util::future::try_join_all;
use indexmap::{IndexMap, IndexSet};

use crate::batch::{DataSource, FetchEnv, FetchKey};

use super::data_cache::DataCache;

type CompleteBatchFn = Box<dyn FnOnce(&mut DataCache)>;

type ExecuteBatchFuture<'a, E> =
    Pin<Box<dyn Future<Output = Result<CompleteBatchFn, <E as FetchEnv>::Error>> + 'a>>;

pub(crate) struct RequestStore<E> {
    batches: IndexMap<TypeId, Box<dyn PendingBatch<E>>>,
}

impl<E> RequestStore<E> {
    pub(crate) fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    pub(crate) fn insert<K>(&mut self, key: &K)
    where
        E: DataSource<K> + 'static,
        K: FetchKey,
    {
        let batch = self
            .batches
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(IndexSet::<K>::new()));
        batch.insert(key);
    }

    pub(crate) async fn execute_round(
        self,
        env: &E,
        data_cache: &mut DataCache,
    ) -> Result<(), E::Error>
    where
        E: FetchEnv,
    {
        let completions =
            try_join_all(self.batches.into_values().map(|batch| batch.execute(env))).await?;

        for complete in completions {
            complete(data_cache);
        }

        Ok(())
    }
}

impl<E> Default for RequestStore<E> {
    fn default() -> Self {
        Self {
            batches: IndexMap::new(),
        }
    }
}

trait PendingBatch<E> {
    fn insert(&mut self, key: &dyn Any);

    fn execute<'a>(self: Box<Self>, env: &'a E) -> ExecuteBatchFuture<'a, E>
    where
        E: FetchEnv + 'a;
}

impl<E, K> PendingBatch<E> for IndexSet<K>
where
    E: DataSource<K>,
    K: FetchKey,
{
    fn insert(&mut self, key: &dyn Any) {
        let key = key
            .downcast_ref::<K>()
            .expect("request store batch type should match fetch key type");
        self.insert(key.clone());
    }

    fn execute<'a>(self: Box<Self>, env: &'a E) -> ExecuteBatchFuture<'a, E>
    where
        E: FetchEnv + 'a,
    {
        let keys = (*self).into_iter().collect::<Vec<_>>();

        Box::pin(async move {
            let mut outputs = env.batch_fetch(&keys).await?;

            Ok(Box::new(move |data_cache: &mut DataCache| {
                for key in keys {
                    let output = outputs
                        .shift_remove(&key)
                        .expect("fetch batch should return exactly one output per request");
                    data_cache.insert(key, output);
                }
            }) as CompleteBatchFn)
        })
    }
}
