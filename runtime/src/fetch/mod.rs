mod computation;
mod continuation;
mod data_cache;
mod request_store;
mod result;
mod state;
mod value;

use std::{future::poll_fn, marker::PhantomData, task::Poll};

use futures_util::{StreamExt, stream::FuturesUnordered};

use crate::{
    FetchError,
    batch::{DataSource, Env, Request},
};

use computation::{Computation, Resumption, Step};
use continuation::Continuation;
use request_store::ExecuteBatchFuture;
use state::FetchState;
use value::Value;

pub struct Fetch<E, A> {
    comp: Computation<E>,
    _output: PhantomData<fn(A) -> A>,
}

impl<E, A> From<Resumption<E>> for Fetch<E, A> {
    fn from(resumption: Resumption<E>) -> Self {
        Self {
            comp: Computation {
                resumption,
                conts: Vec::new(),
            },
            _output: PhantomData,
        }
    }
}

impl<E, A> Fetch<E, A> {
    pub fn pure(value: A) -> Self
    where
        A: 'static,
    {
        Self::from(Resumption::new(|_| Step::Ready(Value::new(value))))
    }

    pub fn map<C>(mut self, map: impl FnOnce(A) -> C + 'static) -> Fetch<E, C>
    where
        A: 'static,
        C: 'static,
    {
        self.comp.conts.push(Continuation::map(map));

        Fetch {
            comp: self.comp,
            _output: PhantomData,
        }
    }

    pub fn zip<C>(self, other: Fetch<E, C>) -> Fetch<E, (A, C)>
    where
        E: 'static,
        A: 'static,
        C: 'static,
    {
        Fetch::from(Resumption::with(
            vec![self.comp, other.comp],
            |state, children| {
                let Ok([left, right]) = <[_; 2]>::try_from(children) else {
                    unreachable!("zip should have exactly two children");
                };
                let right_id = state.schedule(right);
                let left_id = state.schedule(left);

                state.when_ready(left_id, move |state| {
                    let left = state.results.take(left_id).unwrap().into_inner::<A>();

                    state.when_ready(right_id, move |state| {
                        let right = state.results.take(right_id).unwrap().into_inner::<C>();

                        Step::Ready(Value::new((left, right)))
                    })
                })
            },
        ))
    }

    pub fn and_then<C>(mut self, then: impl FnOnce(A) -> Fetch<E, C> + 'static) -> Fetch<E, C>
    where
        A: 'static,
        C: 'static,
    {
        self.comp
            .conts
            .push(Continuation::bind(|value| then(value).comp));

        Fetch {
            comp: self.comp,
            _output: PhantomData,
        }
    }
}

impl<E, A> Fetch<E, A>
where
    E: Env,
    A: 'static,
{
    pub(crate) async fn run_with(self, env: &E) -> Result<A, FetchError<E::Error>> {
        const POLL_BUDGET: usize = 256;

        let mut state = FetchState::new();
        let root_id = state.schedule(self.comp);
        let mut jobs = FuturesUnordered::<ExecuteBatchFuture<'_, E>>::new();

        poll_fn(|cx| {
            let mut budget = POLL_BUDGET;

            while budget > 0 {
                budget -= 1;
                if let Some(job) = state.runnable.pop() {
                    job.run(&mut state, &mut budget);
                    continue;
                }

                if let Some(value) = state.results.take(root_id) {
                    return Poll::Ready(Ok(value.into_inner::<A>()));
                }

                match jobs.poll_next_unpin(cx) {
                    Poll::Ready(Some(Ok(complete))) => {
                        for (rvar, value) in complete {
                            state.complete(rvar, value);
                        }
                    }
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

pub fn fetch<E, R>(req: R) -> Fetch<E, R::Output>
where
    E: DataSource<R>,
    R: Request + 'static,
    R::Output: Clone + 'static,
{
    Fetch::from(Resumption::new(move |state| {
        let id = state.register(req);

        state.when_ready(id, move |state| {
            Step::Ready(Value::new(
                state
                    .results
                    .get(id)
                    .unwrap()
                    .downcast_ref::<R::Output>()
                    .clone(),
            ))
        })
    }))
}

fn collect<E, A: 'static>(
    mut id_stack: Vec<result::ResultId>,
    mut values: Vec<A>,
    state: &mut FetchState<E>,
) -> Step<E> {
    while let Some(&id) = id_stack.last() {
        if let Some(value) = state.results.take(id) {
            id_stack.pop();
            values.push(value.into_inner::<A>());
        } else {
            return Step::Blocked(
                id,
                Resumption::new(move |state| collect(id_stack, values, state)),
            );
        };
    }

    Step::Ready(Value::new(values))
}

pub fn traverse<E, T, C>(
    items: impl IntoIterator<Item = T>,
    fetch: impl Fn(T) -> Fetch<E, C>,
) -> Fetch<E, Vec<C>>
where
    E: 'static,
    C: 'static,
{
    let comps = items
        .into_iter()
        .map(|item| fetch(item).comp)
        .collect::<Vec<_>>();

    if comps.is_empty() {
        return Fetch::pure(Vec::new());
    }

    Fetch::from(Resumption::with(comps, |state, comps| {
        let mut id_stack = Vec::with_capacity(comps.len());
        let values = Vec::<C>::with_capacity(comps.len());

        for comp in comps.into_iter().rev() {
            id_stack.push(state.schedule(comp));
        }

        collect(id_stack, values, state)
    }))
}
