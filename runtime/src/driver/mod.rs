mod data_cache;
mod request_store;
mod result;
mod state;
mod value;

use std::{
    future::{Future, poll_fn},
    ops::AsyncFnOnce,
    pin::pin,
    sync::Mutex,
    task::Poll,
};

use futures_util::{StreamExt, stream::FuturesUnordered};

use crate::{
    FetchError,
    batch::{DataSource, Env, Request},
};

use request_store::ExecuteBatchFuture;
use state::RunState;

pub struct Scope<E: Env> {
    state: Mutex<RunState<E>>,
}

impl<E: Env> Scope<E> {
    fn new() -> Self {
        Self {
            state: Mutex::new(RunState::new()),
        }
    }

    pub fn fetch<R>(
        &self,
        req: R,
    ) -> impl Future<Output = Result<R::Output, FetchError<E::Error>>> + Send + '_
    where
        E: DataSource<R>,
        R: Request + 'static,
        R::Output: Clone + Send + 'static,
    {
        async {
            let id = self.state.lock().unwrap().register(req);
            poll_fn(|cx| self.state.lock().unwrap().results.poll(id, cx.waker())).await
        }
    }
}

pub async fn run<E, A>(env: &E, computation: impl for<'a> AsyncFnOnce(&'a Scope<E>) -> A) -> A
where
    E: Env,
{
    const POLL_BUDGET: usize = 256;

    let scope = Scope::new();
    let mut computation = pin!(computation(&scope));
    let mut jobs = FuturesUnordered::<ExecuteBatchFuture<'_, E>>::new();

    poll_fn(|cx| {
        for _ in 0..POLL_BUDGET {
            if let Poll::Ready(output) = computation.as_mut().poll(cx) {
                return Poll::Ready(output);
            }

            jobs.extend(scope.state.lock().unwrap().reqs.take_jobs(env));
            match jobs.poll_next_unpin(cx) {
                Poll::Ready(Some(complete)) => {
                    let waiters = {
                        let mut state = scope.state.lock().unwrap();
                        complete
                            .into_iter()
                            .flat_map(|(rvar, value)| state.results.complete(rvar, value))
                            .collect::<Vec<_>>()
                    };
                    for waiter in waiters {
                        waiter.wake();
                    }
                }
                Poll::Ready(None) | Poll::Pending => return Poll::Pending,
            }
        }
        cx.waker().wake_by_ref();
        Poll::Pending
    })
    .await
}
