use crate::batch::{DataSource, Request};

use super::{
    computation::{Computation, Resumption, Step},
    continuation::Job,
    data_cache::DataCache,
    request_store::RequestStore,
    result::{ResultId, ResultSlots, ResultVar},
    value::Value,
};

pub(super) struct FetchState<E> {
    data_cache: DataCache,
    pub(super) reqs: RequestStore<E>,
    pub(super) results: ResultSlots<E>,
    pub(super) runnable: Vec<Job<E>>,
}

impl<E> FetchState<E> {
    pub(super) fn new() -> Self {
        Self {
            data_cache: DataCache::default(),
            reqs: RequestStore::new(),
            results: ResultSlots::new(),
            runnable: Vec::new(),
        }
    }

    pub(super) fn register<R>(&mut self, req: R) -> ResultId
    where
        E: DataSource<R>,
        R: Request + 'static,
        R::Output: 'static,
    {
        *self.data_cache.entry(req).or_insert_with_key(|req| {
            let req = req.clone();
            let rvar = self.results.alloc();
            let id = rvar.id();

            self.reqs.insert(req, rvar);

            id
        })
    }

    pub(super) fn when_ready(
        &mut self,
        id: ResultId,
        resume: impl FnOnce(&mut Self) -> Step<E> + 'static,
    ) -> Step<E> {
        if self.results.get(id).is_some() {
            resume(self)
        } else {
            Step::Blocked(id, Resumption::new(resume))
        }
    }

    pub(super) fn complete(&mut self, rvar: ResultVar, value: Value) {
        let mut waiters = self.results.complete(rvar, value);

        while let Some(job) = waiters.pop() {
            self.runnable.push(job);
        }
    }

    pub(super) fn schedule(&mut self, comp: Computation<E>) -> ResultId {
        let rvar = self.results.alloc();
        let id = rvar.id();

        self.runnable.push(Job::new(comp, rvar));

        id
    }
}
