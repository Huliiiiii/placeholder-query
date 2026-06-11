pub(crate) mod data_cache;
mod request_store;

use std::marker::PhantomData;

use crate::batch::{DataSource, FetchEnv, FetchKey};

use data_cache::DataCache;
use request_store::RequestStore;

pub struct FetchCx<E> {
    _env: PhantomData<fn() -> E>,
}

type StepFn<E, A> = Box<dyn FnOnce(&mut FetchState<E>) -> Step<E, A>>;

pub struct Fetch<E, A> {
    step: StepFn<E, A>,
}

struct FetchState<E> {
    data_cache: DataCache,
    requests: RequestStore<E>,
}

enum Step<E, A> {
    Ready(A),
    Blocked(StepFn<E, A>),
}

impl<E, A> Step<E, A> {
    fn resume(self, state: &mut FetchState<E>) -> Self {
        match self {
            Step::Ready(value) => Step::Ready(value),
            Step::Blocked(task) => task(state),
        }
    }
}

impl<E, A> Step<E, A>
where
    E: 'static,
    A: 'static,
{
    fn map<C>(self, map: impl FnOnce(A) -> C + 'static) -> Step<E, C>
    where
        C: 'static,
    {
        match self {
            Step::Ready(value) => Step::Ready(map(value)),
            Step::Blocked(task) => Step::Blocked(Box::new(|state| task(state).map(map))),
        }
    }

    fn and_then<C>(
        self,
        then: impl FnOnce(A, &FetchCx<E>) -> Fetch<E, C> + 'static,
        state: &mut FetchState<E>,
    ) -> Step<E, C>
    where
        C: 'static,
    {
        match self {
            Step::Ready(value) => {
                let cx = FetchCx::new();
                then(value, &cx).poll(state)
            }
            Step::Blocked(task) => {
                Step::Blocked(Box::new(|state| task(state).and_then(then, state)))
            }
        }
    }

    fn zip<C>(self, other: Step<E, C>) -> Step<E, (A, C)>
    where
        C: 'static,
    {
        match (self, other) {
            (Step::Ready(left), Step::Ready(right)) => Step::Ready((left, right)),
            (left, right) => Step::Blocked(Box::new(|state| {
                left.resume(state).zip(right.resume(state))
            })),
        }
    }

    fn collect(items: Vec<Self>) -> Step<E, Vec<A>> {
        if items.iter().all(|item| matches!(item, Step::Ready(_))) {
            Step::Ready(
                items
                    .into_iter()
                    .map(|item| match item {
                        Step::Ready(value) => value,
                        Step::Blocked(_) => unreachable!("all traverse items should be ready"),
                    })
                    .collect(),
            )
        } else {
            Step::Blocked(Box::new(|state| {
                let items = items.into_iter().map(|item| item.resume(state)).collect();

                Step::collect(items)
            }))
        }
    }
}

impl<E, A> Fetch<E, A> {
    pub fn new(build: impl FnOnce(&FetchCx<E>) -> Fetch<E, A>) -> Self {
        let cx = FetchCx::new();

        build(&cx)
    }

    pub fn pure(value: A) -> Self
    where
        A: 'static,
    {
        Self::from_step_fn(|_| Step::Ready(value))
    }

    pub fn map<C>(self, map: impl FnOnce(A) -> C + 'static) -> Fetch<E, C>
    where
        E: 'static,
        A: 'static,
        C: 'static,
    {
        Fetch::from_step_fn(|state| self.poll(state).map(map))
    }

    pub fn zip<C>(self, other: Fetch<E, C>) -> Fetch<E, (A, C)>
    where
        E: 'static,
        A: 'static,
        C: 'static,
    {
        Fetch::from_step_fn(|state| self.poll(state).zip(other.poll(state)))
    }

    pub fn and_then<C>(
        self,
        then: impl FnOnce(A, &FetchCx<E>) -> Fetch<E, C> + 'static,
    ) -> Fetch<E, C>
    where
        E: 'static,
        A: 'static,
        C: 'static,
    {
        Fetch::from_step_fn(|state| self.poll(state).and_then(then, state))
    }

    fn from_step_fn(step: impl FnOnce(&mut FetchState<E>) -> Step<E, A> + 'static) -> Self {
        Self {
            step: Box::new(step),
        }
    }

    fn poll(self, state: &mut FetchState<E>) -> Step<E, A> {
        (self.step)(state)
    }
}

impl<E, A> Fetch<E, A>
where
    E: FetchEnv + 'static,
{
    pub async fn run(self, env: &E) -> Result<A, E::Error> {
        let mut state = FetchState {
            data_cache: DataCache::default(),
            requests: RequestStore::default(),
        };
        let mut step = self.poll(&mut state);

        loop {
            match step {
                Step::Ready(value) => return Ok(value),
                Step::Blocked(task) => {
                    let requests = std::mem::take(&mut state.requests);
                    assert!(
                        !requests.is_empty(),
                        "fetch made no progress while waiting for a round"
                    );
                    requests.execute_round(env, &mut state.data_cache).await?;
                    step = task(&mut state);
                }
            }
        }
    }
}

impl<E> FetchCx<E> {
    fn new() -> Self {
        Self { _env: PhantomData }
    }

    pub fn fetch<K>(&self, key: K) -> Fetch<E, K::Output>
    where
        E: DataSource<K> + 'static,
        K: FetchKey,
    {
        Fetch::from_step_fn(move |state| {
            if let Some(value) = state.data_cache.get(&key) {
                return Step::Ready(value);
            }

            state.requests.insert(&key);
            Step::Blocked(Box::new(move |state| {
                Step::Ready(
                    state
                        .data_cache
                        .get(&key)
                        .expect("fetch key should be available after request round"),
                )
            }))
        })
    }

    pub fn traverse<T, C>(
        &self,
        items: impl IntoIterator<Item = T>,
        fetch: impl Fn(T, &FetchCx<E>) -> Fetch<E, C>,
    ) -> Fetch<E, Vec<C>>
    where
        E: 'static,
        C: 'static,
    {
        let fetches = items
            .into_iter()
            .map(|item| fetch(item, self))
            .collect::<Vec<_>>();

        if fetches.is_empty() {
            return Fetch::pure(Vec::new());
        }

        Fetch::from_step_fn(|state| {
            let items = fetches
                .into_iter()
                .map(|fetch| fetch.poll(state))
                .collect::<Vec<_>>();

            Step::collect(items)
        })
    }
}
