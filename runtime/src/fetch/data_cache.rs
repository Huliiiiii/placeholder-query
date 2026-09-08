use std::{
    any::{Any, TypeId},
    collections::{HashMap, hash_map::Entry},
    hash::Hash,
};

use super::result::ResultId;

type Bucket<R> = HashMap<R, ResultId>;

#[derive(Default)]
pub(super) struct DataCache {
    buckets: HashMap<TypeId, Box<dyn Any>>,
}

impl DataCache {
    pub(super) fn entry<R>(&mut self, req: R) -> Entry<'_, R, ResultId>
    where
        R: Eq + Hash + 'static,
    {
        let bucket = self
            .buckets
            .entry(TypeId::of::<R>())
            .or_insert_with(|| Box::new(Bucket::<R>::new()));

        // SAFETY: buckets are private, so it's safe to assume the type of the bucket is correct.
        let values = unsafe { bucket.downcast_mut::<Bucket<R>>().unwrap_unchecked() };

        values.entry(req)
    }
}
