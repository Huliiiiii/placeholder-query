use std::{
    any::{Any, TypeId},
    marker::PhantomData,
};

use indexmap::IndexMap;

use crate::batch::{FetchBackend, FetchKey};

pub(crate) struct DataCache<B>
where
    B: FetchBackend,
{
    buckets: IndexMap<TypeId, Box<dyn Any>>,
    _builder: PhantomData<fn() -> B>,
}

impl<B> DataCache<B>
where
    B: FetchBackend,
{
    pub(crate) fn new() -> Self {
        Self {
            buckets: IndexMap::new(),
            _builder: PhantomData,
        }
    }

    pub(crate) fn get<K>(&self, key: &K) -> Option<K::Output>
    where
        K: FetchKey<B>,
    {
        let bucket = self.buckets.get(&TypeId::of::<K>())?;
        let bucket = bucket
            .downcast_ref::<KeyCacheBucket<K, K::Output>>()
            .expect("data cache bucket type should match fetch key type");

        bucket.values.get(key).cloned()
    }

    pub(crate) fn insert<K>(&mut self, key: K, value: K::Output)
    where
        K: FetchKey<B>,
    {
        let bucket = self
            .buckets
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(KeyCacheBucket::<K, K::Output>::new()));
        let bucket = bucket
            .downcast_mut::<KeyCacheBucket<K, K::Output>>()
            .expect("data cache bucket type should match fetch key type");

        bucket.values.insert(key, value);
    }
}

struct KeyCacheBucket<K, O> {
    values: IndexMap<K, O>,
}

impl<K, O> KeyCacheBucket<K, O> {
    fn new() -> Self {
        Self {
            values: IndexMap::new(),
        }
    }
}
