use std::{
    any::{Any, TypeId},
    hash::Hash,
};

use indexmap::{IndexMap, IndexSet};

use crate::batch::{FetchBackend, FetchBatch, FetchKey};

use super::{AsyncFetchExecutor, data_cache::DataCache};

type CompleteFn<B> = dyn FnOnce(
    Vec<<B as FetchBackend>::Row>,
    &mut DataCache<B>,
) -> Result<(), <B as FetchBackend>::Error>;

pub(crate) struct RequestStore<B>
where
    B: FetchBackend,
{
    batches: IndexMap<TypeId, Box<dyn PendingBatch<B>>>,
}

impl<B> RequestStore<B>
where
    B: FetchBackend + 'static,
{
    pub(crate) fn new() -> Self {
        Self {
            batches: IndexMap::new(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    pub(crate) fn insert<K>(&mut self, key: K)
    where
        K: FetchKey<B>,
    {
        let batch = self
            .batches
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(PendingKeySet::<K>::new()));
        batch.insert(Box::new(key));
    }

    pub(crate) fn execute_round<E>(
        self,
        execute_round: &mut impl FnMut(Vec<B::Request>) -> Result<Vec<Vec<B::Row>>, E>,
        data_cache: &mut DataCache<B>,
    ) -> Result<(), E>
    where
        E: From<B::Error>,
    {
        let mut completions = Vec::<Box<CompleteFn<B>>>::new();
        let mut requests = Vec::new();

        for (_, batch) in self.batches {
            let (request, complete) = batch.prepare().into_parts();
            requests.push(request);
            completions.push(complete);
        }

        let rows = execute_round(requests)?;

        assert_eq!(
            rows.len(),
            completions.len(),
            "execute_round should return exactly one row set per request"
        );

        for (complete, rows) in completions.into_iter().zip(rows) {
            complete(rows, data_cache).map_err(E::from)?;
        }

        Ok(())
    }

    pub(crate) async fn execute_round_async<E>(
        self,
        executor: &mut impl AsyncFetchExecutor<B, Error = E>,
        data_cache: &mut DataCache<B>,
    ) -> Result<(), E>
    where
        E: From<B::Error>,
    {
        let mut completions = Vec::<Box<CompleteFn<B>>>::new();
        let mut requests = Vec::new();

        for (_, batch) in self.batches {
            let (request, complete) = batch.prepare().into_parts();
            requests.push(request);
            completions.push(complete);
        }

        let rows = AsyncFetchExecutor::execute_round(executor, requests).await?;

        assert_eq!(
            rows.len(),
            completions.len(),
            "execute_round should return exactly one row set per request"
        );

        for (complete, rows) in completions.into_iter().zip(rows) {
            complete(rows, data_cache).map_err(E::from)?;
        }

        Ok(())
    }
}

trait PendingBatch<B>
where
    B: FetchBackend,
{
    fn insert(&mut self, key: Box<dyn Any>);

    fn prepare(self: Box<Self>) -> PreparedRequest<B>;
}

struct PreparedRequest<B>
where
    B: FetchBackend,
{
    request: B::Request,
    complete: Box<CompleteFn<B>>,
}

impl<B> PreparedRequest<B>
where
    B: FetchBackend,
{
    fn new(
        request: B::Request,
        complete: impl FnOnce(Vec<B::Row>, &mut DataCache<B>) -> Result<(), B::Error> + 'static,
    ) -> Self {
        Self {
            request,
            complete: Box::new(complete),
        }
    }

    fn into_parts(self) -> (B::Request, Box<CompleteFn<B>>) {
        (self.request, self.complete)
    }
}

struct PendingKeySet<K> {
    keys: IndexSet<K>,
}

impl<K> PendingKeySet<K> {
    fn new() -> Self {
        Self {
            keys: IndexSet::new(),
        }
    }
}

impl<K> PendingKeySet<K>
where
    K: Eq + Hash,
{
    fn insert(&mut self, key: K) {
        self.keys.insert(key);
    }
}

impl<B, K> PendingBatch<B> for PendingKeySet<K>
where
    B: FetchBackend + 'static,
    K: FetchKey<B>,
{
    // TODO: Remove box when std can downcast by reference
    fn insert(&mut self, key: Box<dyn Any>) {
        let key = *key
            .downcast::<K>()
            .expect("request store batch type should match fetch key type");
        self.insert(key);
    }

    fn prepare(self: Box<Self>) -> PreparedRequest<B> {
        let keys = self.keys.iter().cloned().collect::<Vec<_>>();
        let request: FetchBatch<B, K> = K::batch(&keys).into();

        PreparedRequest::new(request.request, move |rows, data_cache| {
            let mut outputs = (request.collect)(rows)?;

            for key in keys {
                let output = outputs
                    .shift_remove(&key)
                    .expect("fetch batch should return exactly one output per request");
                data_cache.insert(key, output);
            }

            Ok(())
        })
    }
}
