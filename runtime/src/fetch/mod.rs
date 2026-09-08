mod continuation;
mod data_cache;
mod request_store;
mod result;
mod run_queue;

use std::{collections::hash_map::Entry, future::poll_fn, marker::PhantomData, task::Poll, vec};

use futures_util::{StreamExt, stream::FuturesUnordered};

use crate::{
    FetchError,
    batch::{DataSource, FetchEnv, Request},
};

use continuation::{Computation, Job, Step, StepFn, Value};
use data_cache::DataCache;
use request_store::{ExecuteBatchFuture, RequestStore};
use result::{ResultId, ResultSlot, ResultSlots};
use run_queue::RunQueue;

pub struct Fetch<E, A> {
    computation: Computation<E>,
    _output: PhantomData<fn() -> A>,
}

#[derive_where::derive_where(Default)]
struct FetchState<E> {
    data_cache: DataCache,
    reqs: RequestStore<E>,
    results: ResultSlots<E>,
    runnable: RunQueue<E>,
}

impl<E> FetchState<E> {
    fn when_ready(&mut self, id: ResultId, resume: StepFn<E>) -> Step<'_, E> {
        match &mut self.results[id] {
            ResultSlot::Pending(waiters) => Step::Blocked(waiters, resume),
            ResultSlot::Ready(_) => resume(self),
            ResultSlot::Taken => unreachable!("cannot wait for a consumed result"),
        }
    }

    fn complete(&mut self, id: ResultId, value: Value) {
        let ResultSlot::Pending(waiters) =
            std::mem::replace(&mut self.results[id], ResultSlot::Ready(value))
        else {
            unreachable!("a result can only be completed once")
        };

        self.runnable.prepend(waiters);
    }

    fn get<A: 'static>(&self, id: ResultId) -> &A {
        let ResultSlot::Ready(value) = &self.results[id] else {
            unreachable!("only ready results can be read")
        };

        value
            .downcast_ref()
            .expect("result type should match its computation")
    }

    fn take<A: 'static>(&mut self, id: ResultId) -> A {
        let ResultSlot::Ready(value) = std::mem::replace(&mut self.results[id], ResultSlot::Taken)
        else {
            unreachable!("a computation result can only be taken once")
        };

        *value
            .downcast()
            .expect("result type should match its computation")
    }

    fn take_ready<A: 'static>(&mut self, id: ResultId) -> Option<A> {
        if self.results[id].is_ready() {
            Some(self.take(id))
        } else {
            None
        }
    }
}

impl<E, A> Fetch<E, A> {
    pub fn pure(value: A) -> Self
    where
        A: 'static,
    {
        Self::from_step_fn(|_| Step::ready(value))
    }

    pub fn map<C>(self, map: impl FnOnce(A) -> C + 'static) -> Fetch<E, C>
    where
        A: 'static,
        C: 'static,
    {
        let mut computation = self.computation;
        computation.conts.push(Box::new(|value| {
            let value = *value
                .downcast::<A>()
                .expect("map input should match fetch output");
            Fetch::pure(map(value)).computation
        }));

        Fetch {
            computation,
            _output: PhantomData,
        }
    }

    pub fn zip<C>(self, other: Fetch<E, C>) -> Fetch<E, (A, C)>
    where
        E: 'static,
        A: 'static,
        C: 'static,
    {
        Fetch::from_step_fn(|state| {
            let left_id = state.results.alloc();
            let right_id = state.results.alloc();
            state.runnable.prepend([
                Job::new(self.computation, left_id),
                Job::new(other.computation, right_id),
            ]);

            state.when_ready(
                left_id,
                Box::new(move |state| {
                    let left = state.take::<A>(left_id);
                    state.when_ready(
                        right_id,
                        Box::new(move |state| Step::ready((left, state.take::<C>(right_id)))),
                    )
                }),
            )
        })
    }

    pub fn and_then<C>(self, then: impl FnOnce(A) -> Fetch<E, C> + 'static) -> Fetch<E, C>
    where
        A: 'static,
        C: 'static,
    {
        let mut computation = self.computation;
        computation.conts.push(Box::new(|value| {
            let value = *value
                .downcast::<A>()
                .expect("bind input should match fetch output");
            then(value).computation
        }));

        Fetch {
            computation,
            _output: PhantomData,
        }
    }

    fn from_step_fn(
        step: impl for<'a> FnOnce(&'a mut FetchState<E>) -> Step<'a, E> + 'static,
    ) -> Self {
        Self {
            computation: Computation {
                step: Box::new(step),
                conts: Vec::new(),
            },
            _output: PhantomData,
        }
    }
}

impl<E, A> Fetch<E, A>
where
    E: FetchEnv,
    A: 'static,
{
    pub(crate) async fn run_with(self, env: &E) -> Result<A, FetchError<E::Error>> {
        const POLL_BUDGET: usize = 256;

        let mut state = FetchState::default();
        let root_id = state.results.alloc();
        state
            .runnable
            .prepend([Job::new(self.computation, root_id)]);
        let mut jobs = FuturesUnordered::<ExecuteBatchFuture<'_, E>>::new();

        poll_fn(|cx| {
            for _ in 0..POLL_BUDGET {
                if let Some(job) = state.runnable.next() {
                    job.run(&mut state);
                    continue;
                }

                if let Some(value) = state.take_ready::<A>(root_id) {
                    return Poll::Ready(Ok(value));
                }

                match jobs.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(complete))) => complete(&mut state),
                    Poll::Ready(Some(Err(error))) => return Poll::Ready(Err(error)),
                    Poll::Ready(None) | Poll::Pending => {
                        let pending = state.reqs.take_jobs(env);
                        if pending.is_empty() {
                            assert!(
                                !jobs.is_empty(),
                                "fetch made no progress while waiting for requests"
                            );
                            return Poll::Pending;
                        }

                        jobs.extend(pending);
                    }
                }
            }

            cx.waker().wake_by_ref();
            Poll::Pending
        })
        .await
    }
}

fn collect<E, A: 'static>(
    mut ids: vec::IntoIter<ResultId>,
    mut values: Vec<A>,
    state: &mut FetchState<E>,
) -> Step<'_, E> {
    for id in ids.by_ref() {
        if let Some(value) = state.take_ready::<A>(id) {
            values.push(value);
        } else {
            return state.when_ready(
                id,
                Box::new(move |state| {
                    values.push(state.take::<A>(id));
                    collect(ids, values, state)
                }),
            );
        }
    }

    Step::ready(values)
}

pub fn fetch<E, R>(req: R) -> Fetch<E, R::Output>
where
    E: DataSource<R>,
    R: Request + Clone + 'static,
    R::Output: Clone + 'static,
{
    Fetch::from_step_fn(move |state| {
        let id = match state.data_cache.entry(req.clone()) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let id = state.results.alloc();
                entry.insert(id);
                state.reqs.insert(req, id);
                id
            }
        };

        state.when_ready(
            id,
            Box::new(move |state| Step::ready(state.get::<R::Output>(id).clone())),
        )
    })
}

pub fn traverse<E, T, C>(
    items: impl IntoIterator<Item = T>,
    fetch: impl Fn(T) -> Fetch<E, C>,
) -> Fetch<E, Vec<C>>
where
    E: 'static,
    C: 'static,
{
    let fetches = items.into_iter().map(fetch).collect::<Vec<_>>();

    if fetches.is_empty() {
        return Fetch::pure(Vec::new());
    }

    Fetch::from_step_fn(|state| {
        let ids = fetches
            .iter()
            .map(|_| state.results.alloc())
            .collect::<Vec<_>>();
        state.runnable.prepend(
            fetches
                .into_iter()
                .zip(ids.iter().copied())
                .map(|(fetch, id)| Job::new(fetch.computation, id)),
        );

        collect::<E, C>(ids.into_iter(), Vec::new(), state)
    })
}
