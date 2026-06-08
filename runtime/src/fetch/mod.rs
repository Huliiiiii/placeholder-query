pub(crate) mod data_cache;
mod request_store;

use std::{future::Future, marker::PhantomData};

use crate::batch::{FetchBackend, FetchKey};

use data_cache::DataCache;
use request_store::RequestStore;

pub struct FetchCx<B>
where
    B: FetchBackend,
{
    _builder: PhantomData<fn() -> B>,
}

type StepFn<B, A> = Box<dyn FnOnce(&mut FetchState<B>) -> FetchStep<B, A>>;

pub struct Fetch<B, A>
where
    B: FetchBackend,
{
    step: StepFn<B, A>,
}

struct FetchState<B>
where
    B: FetchBackend,
{
    data_cache: DataCache<B>,
    requests: RequestStore<B>,
}

enum FetchStep<B, A>
where
    B: FetchBackend,
{
    Ready(A),
    Pending(Fetch<B, A>),
}

pub trait FetchExecutor<B>
where
    B: FetchBackend,
{
    type Error;

    fn execute_round(&mut self, requests: Vec<B::Request>)
    -> Result<Vec<Vec<B::Row>>, Self::Error>;
}

pub trait AsyncFetchExecutor<B>
where
    B: FetchBackend,
{
    type Error;

    fn execute_round(
        &mut self,
        requests: Vec<B::Request>,
    ) -> impl Future<Output = Result<Vec<Vec<B::Row>>, Self::Error>> + '_;
}

impl<B, A> Fetch<B, A>
where
    B: FetchBackend + 'static,
{
    pub fn new(build: impl FnOnce(&FetchCx<B>) -> Fetch<B, A>) -> Self {
        let cx = FetchCx {
            _builder: PhantomData,
        };

        build(&cx)
    }

    pub fn pure(value: A) -> Self
    where
        A: 'static,
    {
        Self::from_step(move |_| FetchStep::Ready(value))
    }

    pub fn map<C>(self, map: impl FnOnce(A) -> C + 'static) -> Fetch<B, C>
    where
        A: 'static,
        C: 'static,
    {
        Fetch::from_step(move |state| match self.poll(state) {
            FetchStep::Ready(value) => FetchStep::Ready(map(value)),
            FetchStep::Pending(fetch) => FetchStep::Pending(fetch.map(map)),
        })
    }

    pub fn zip<C>(self, other: Fetch<B, C>) -> Fetch<B, (A, C)>
    where
        A: 'static,
        C: 'static,
    {
        Fetch::from_step(move |state| {
            let left = self.poll(state);
            let right = other.poll(state);

            match (left, right) {
                (FetchStep::Ready(left), FetchStep::Ready(right)) => {
                    FetchStep::Ready((left, right))
                }
                (FetchStep::Pending(left), FetchStep::Ready(right)) => {
                    FetchStep::Pending(left.zip(Fetch::pure(right)))
                }
                (FetchStep::Ready(left), FetchStep::Pending(right)) => {
                    FetchStep::Pending(Fetch::pure(left).zip(right))
                }
                (FetchStep::Pending(left), FetchStep::Pending(right)) => {
                    FetchStep::Pending(left.zip(right))
                }
            }
        })
    }

    pub fn and_then<C>(
        self,
        then: impl FnOnce(A, &FetchCx<B>) -> Fetch<B, C> + 'static,
    ) -> Fetch<B, C>
    where
        A: 'static,
        C: 'static,
    {
        Fetch::from_step(|state| match self.poll(state) {
            FetchStep::Ready(value) => {
                let cx = FetchCx {
                    _builder: PhantomData,
                };
                then(value, &cx).poll(state)
            }
            FetchStep::Pending(fetch) => FetchStep::Pending(fetch.and_then(then)),
        })
    }

    fn from_step(step: impl FnOnce(&mut FetchState<B>) -> FetchStep<B, A> + 'static) -> Self {
        Self {
            step: Box::new(step),
        }
    }

    fn poll(self, state: &mut FetchState<B>) -> FetchStep<B, A> {
        (self.step)(state)
    }
}

impl<B, A> Fetch<B, A>
where
    B: FetchBackend + 'static,
{
    pub fn run<E>(self, executor: &mut impl FetchExecutor<B, Error = E>) -> Result<A, E>
    where
        E: From<B::Error>,
    {
        self.run_with_executor(executor)
    }

    pub async fn run_async<E>(
        self,
        executor: &mut impl AsyncFetchExecutor<B, Error = E>,
    ) -> Result<A, E>
    where
        E: From<B::Error>,
    {
        let mut state = FetchState::new();
        let mut fetch = self;

        loop {
            match fetch.poll(&mut state) {
                FetchStep::Ready(value) => return Ok(value),
                FetchStep::Pending(next_fetch) => {
                    let requests = state.take_requests();
                    assert!(
                        !requests.is_empty(),
                        "fetch made no progress while waiting for a round"
                    );
                    requests
                        .execute_round_async(executor, &mut state.data_cache)
                        .await?;
                    fetch = next_fetch;
                }
            }
        }
    }

    pub fn run_with<E>(
        self,
        execute_round: impl FnMut(Vec<B::Request>) -> Result<Vec<Vec<B::Row>>, E>,
    ) -> Result<A, E>
    where
        E: From<B::Error>,
    {
        self.run_loop(execute_round)
    }

    fn run_with_executor<E>(self, executor: &mut impl FetchExecutor<B, Error = E>) -> Result<A, E>
    where
        E: From<B::Error>,
    {
        self.run_loop(|requests| executor.execute_round(requests))
    }

    fn run_loop<E>(
        self,
        mut execute_round: impl FnMut(Vec<B::Request>) -> Result<Vec<Vec<B::Row>>, E>,
    ) -> Result<A, E>
    where
        E: From<B::Error>,
    {
        let mut state = FetchState::new();
        let mut fetch = self;

        loop {
            match fetch.poll(&mut state) {
                FetchStep::Ready(value) => return Ok(value),
                FetchStep::Pending(next_fetch) => {
                    let requests = state.take_requests();
                    assert!(
                        !requests.is_empty(),
                        "fetch made no progress while waiting for a round"
                    );
                    requests.execute_round(&mut execute_round, &mut state.data_cache)?;
                    fetch = next_fetch;
                }
            }
        }
    }
}

impl<B> FetchCx<B>
where
    B: FetchBackend + 'static,
{
    pub fn fetch<K>(&self, key: K) -> Fetch<B, K::Output>
    where
        K: FetchKey<B>,
    {
        fetch_key(key)
    }

    pub fn traverse<T, C>(
        &self,
        items: impl IntoIterator<Item = T>,
        fetch: impl Fn(T, &FetchCx<B>) -> Fetch<B, C>,
    ) -> Fetch<B, Vec<C>>
    where
        C: 'static,
    {
        items
            .into_iter()
            .fold(Fetch::pure(Vec::new()), |acc, item| {
                acc.zip(fetch(item, self)).map(|(mut items, item)| {
                    items.push(item);
                    items
                })
            })
    }
}

impl<B> FetchState<B>
where
    B: FetchBackend + 'static,
{
    fn new() -> Self {
        Self {
            data_cache: DataCache::new(),
            requests: RequestStore::new(),
        }
    }

    fn take_requests(&mut self) -> RequestStore<B> {
        std::mem::replace(&mut self.requests, RequestStore::new())
    }
}

fn fetch_key<B, K>(key: K) -> Fetch<B, K::Output>
where
    B: FetchBackend + 'static,
    K: FetchKey<B>,
{
    Fetch::from_step(move |state| {
        if let Some(value) = state.data_cache.get(&key) {
            return FetchStep::Ready(value);
        }

        state.requests.insert(key.clone());
        FetchStep::Pending(fetch_key(key))
    })
}
