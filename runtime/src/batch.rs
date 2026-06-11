use std::{future::Future, hash::Hash};

use indexmap::IndexMap;

pub trait FetchKey: Clone + Eq + Hash + 'static {
    type Output: Clone + 'static;
}

pub trait FetchEnv {
    type Error;
}

pub trait DataSource<K>: FetchEnv
where
    K: FetchKey,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [K],
    ) -> impl Future<Output = Result<IndexMap<K, K::Output>, Self::Error>> + 'a;
}
