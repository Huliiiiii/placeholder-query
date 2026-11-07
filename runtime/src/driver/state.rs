use crate::{
    Env,
    batch::{DataSource, Request},
};

use super::{
    data_cache::DataCache,
    request_store::RequestStore,
    result::{ResultId, ResultSlots},
};

pub(super) struct RunState<E: Env> {
    data_cache: DataCache,
    pub(super) reqs: RequestStore<E>,
    pub(super) results: ResultSlots<E::Error>,
}

impl<E: Env> RunState<E> {
    pub(super) fn new() -> Self {
        Self {
            data_cache: DataCache::default(),
            reqs: RequestStore::new(),
            results: ResultSlots::new(),
        }
    }

    pub(super) fn register<R>(&mut self, req: R) -> ResultId
    where
        E: DataSource<R>,
        R: Request + 'static,
        R::Output: Send + 'static,
    {
        *self.data_cache.entry(req).or_insert_with_key(|req| {
            let rvar = self.results.alloc();
            let id = rvar.id();
            self.reqs.insert(req.clone(), rvar);
            id
        })
    }
}
