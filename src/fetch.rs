use std::{any::TypeId, collections::HashMap, marker::PhantomData};

use crate::batch::{Batch, BatchNode, FetchKey, FetchKeyNode, PlannedQuery, QueryBuilder};

pub struct FetchCx<B>
where
    B: QueryBuilder,
{
    _builder: PhantomData<fn() -> B>,
}

pub struct Fetch<B>
where
    B: QueryBuilder,
{
    requests: Vec<Box<dyn FetchKeyNode<B>>>,
}

impl<B> Fetch<B>
where
    B: QueryBuilder,
{
    pub fn new(build: impl FnOnce(&FetchCx<B>) -> Fetch<B>) -> Self {
        let cx = FetchCx {
            _builder: PhantomData,
        };

        build(&cx)
    }

    pub fn empty() -> Self {
        Self {
            requests: Vec::new(),
        }
    }

    fn key<K>(key: K) -> Self
    where
        B: Batch<K>,
        K: FetchKey,
    {
        Self {
            requests: vec![Box::new(key)],
        }
    }

    pub fn zip(mut self, other: Fetch<B>) -> Fetch<B> {
        self.requests.extend(other.requests);

        Fetch {
            requests: self.requests,
        }
    }

    pub fn to_queries(&self, builder: &B) -> Vec<PlannedQuery<B::Query>> {
        self.plan_batches()
            .into_iter()
            .map(|batch| batch.build_query(builder))
            .collect()
    }

    pub fn traverse<A>(
        items: impl IntoIterator<Item = A>,
        fetch: impl Fn(A) -> Fetch<B>,
    ) -> Fetch<B> {
        let mut requests = Vec::new();
        for item in items {
            let fetch = fetch(item);
            requests.extend(fetch.requests);
        }

        Fetch { requests }
    }

    pub(crate) fn plan_batches(&self) -> Vec<Box<dyn BatchNode<B>>> {
        let mut batches: Vec<Box<dyn BatchNode<B>>> = Vec::new();
        let mut batch_index: HashMap<TypeId, usize> = HashMap::new();

        for key in &self.requests {
            let type_id = key.key_type_id();
            let index = match batch_index.get(&type_id).copied() {
                Some(index) => index,
                None => {
                    let index = batches.len();
                    batches.push(key.new_batch());
                    batch_index.insert(type_id, index);
                    index
                }
            };

            batches[index].insert(key.as_ref());
        }

        batches
    }
}

impl<B> FetchCx<B>
where
    B: QueryBuilder,
{
    pub fn get<K>(&self, key: K) -> Fetch<B>
    where
        B: Batch<K>,
        K: FetchKey,
    {
        Fetch::key(key)
    }
}
