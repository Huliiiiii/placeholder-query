use std::hash::Hash;

use indexmap::IndexMap;

type KeyCollectFn<B, K> =
    dyn FnOnce(
        Vec<<B as FetchBackend>::Row>,
    ) -> Result<IndexMap<K, <K as FetchKey<B>>::Output>, <B as FetchBackend>::Error>;

pub trait FetchBackend {
    type Request;
    type Row;
    type Error;
}

pub trait FetchKey<B>: Clone + Eq + Hash + 'static
where
    B: FetchBackend,
{
    type Output: Clone + 'static;

    fn batch(keys: &[Self]) -> impl Into<FetchBatch<B, Self>>;
}

pub struct FetchBatch<B, K>
where
    B: FetchBackend,
    K: FetchKey<B>,
{
    pub(crate) request: B::Request,
    pub(crate) collect: Box<KeyCollectFn<B, K>>,
}

impl<B, K> FetchBatch<B, K>
where
    B: FetchBackend,
    K: FetchKey<B>,
{
    pub fn new(
        request: B::Request,
        collect: impl FnOnce(Vec<B::Row>) -> Result<IndexMap<K, K::Output>, B::Error> + 'static,
    ) -> Self {
        Self {
            request,
            collect: Box::new(collect),
        }
    }
}
