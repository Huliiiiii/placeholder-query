use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::batch::FetchKey;

type Bucket<K> = HashMap<K, <K as FetchKey>::Output>;

pub(crate) struct DataCache {
    buckets: HashMap<TypeId, Box<dyn Any>>,
}

impl DataCache {
    pub(crate) fn get<K>(&self, key: &K) -> Option<K::Output>
    where
        K: FetchKey,
    {
        let bucket = self.buckets.get(&TypeId::of::<K>())?;
        let values = bucket
            .downcast_ref::<Bucket<K>>()
            .expect("data cache bucket type should match fetch key type");

        values.get(key).cloned()
    }

    pub(crate) fn insert<K>(&mut self, key: K, value: K::Output)
    where
        K: FetchKey,
    {
        let bucket = self
            .buckets
            .entry(TypeId::of::<K>())
            .or_insert_with(|| Box::new(Bucket::<K>::new()));
        let values = bucket
            .downcast_mut::<Bucket<K>>()
            .expect("data cache bucket type should match fetch key type");

        values.insert(key, value);
    }
}

impl Default for DataCache {
    fn default() -> Self {
        Self {
            buckets: HashMap::new(),
        }
    }
}
