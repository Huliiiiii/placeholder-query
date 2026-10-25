mod support;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    future::{Future, poll_fn},
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};

use placeholder_query_runtime::{DataSource, Env, Fetch, Request, fetch, traverse};

use support::executor::{TestWake, run};

#[derive(Default)]
struct Gate {
    ready: Cell<bool>,
    waker: RefCell<Option<Waker>>,
}

impl Gate {
    async fn wait(&self) {
        poll_fn(|cx| {
            if self.ready.get() {
                Poll::Ready(())
            } else {
                self.waker.borrow_mut().replace(cx.waker().clone());

                Poll::Pending
            }
        })
        .await
    }

    fn release(&self) {
        self.ready.set(true);

        if let Some(waker) = self.waker.borrow_mut().take() {
            waker.wake();
        }
    }
}

#[derive(Default)]
struct SchedulingContext {
    fast_batches: RefCell<Vec<HashMap<FastValue, i32>>>,
    slow_batches: RefCell<Vec<HashMap<SlowValue, i32>>>,
    fast: Gate,
    slow: Gate,
}

impl Env for SchedulingContext {
    type Error = ();
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FastValue(i32);

impl Request for FastValue {
    type Output = i32;
}

impl DataSource<FastValue> for SchedulingContext {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a FastValue>,
    ) -> Result<HashMap<FastValue, i32>, ()> {
        let mut results = HashMap::new();
        for req in reqs {
            assert!(results.insert(req.clone(), req.0).is_none());
        }
        self.fast_batches.borrow_mut().push(results.clone());
        self.fast.wait().await;

        Ok(results)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SlowValue(i32);

impl Request for SlowValue {
    type Output = i32;
}

impl DataSource<SlowValue> for SchedulingContext {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a SlowValue>,
    ) -> Result<HashMap<SlowValue, i32>, ()> {
        let mut results = HashMap::new();
        for req in reqs {
            assert!(results.insert(req.clone(), req.0).is_none());
        }
        self.slow_batches.borrow_mut().push(results.clone());
        self.slow.wait().await;

        Ok(results)
    }
}

#[test]
fn dependent_requests_do_not_wait_for_unrelated_batches() {
    let context = SchedulingContext::default();
    let fetch = traverse([1, 2, 1], |id| fetch(FastValue(id)))
        .and_then(|values| traverse(values, |value| fetch(FastValue(value + 10))))
        .zip(traverse([99], |id| fetch(SlowValue(id))));
    let wake = TestWake::new();
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(context.run(fetch));
    let deadline = Instant::now() + Duration::from_secs(10);

    assert!(future.as_mut().poll(&mut cx).is_pending());
    while wake.take_notification() {
        assert!(Instant::now() < deadline, "test future exceeded 10 seconds");
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }

    assert_eq!(
        *context.fast_batches.borrow(),
        [HashMap::from([(FastValue(1), 1), (FastValue(2), 2)])]
    );
    assert_eq!(
        *context.slow_batches.borrow(),
        [HashMap::from([(SlowValue(99), 99)])]
    );

    context.fast.release();

    while wake.take_notification() {
        assert!(Instant::now() < deadline, "test future exceeded 10 seconds");
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }

    assert_eq!(
        *context.fast_batches.borrow(),
        [
            HashMap::from([(FastValue(1), 1), (FastValue(2), 2)]),
            HashMap::from([(FastValue(11), 11), (FastValue(12), 12)]),
        ]
    );
    assert_eq!(
        *context.slow_batches.borrow(),
        [HashMap::from([(SlowValue(99), 99)])]
    );

    context.slow.release();

    let result = run(future);

    assert_eq!(result.unwrap(), (vec![11, 12, 11], vec![99]));
}

fn assert_yield_preserves_batching(chain: Fetch<SchedulingContext, i32>, expected: i32) {
    let context = SchedulingContext::default();
    context.fast.release();

    let wake = TestWake::new();
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(
        context.run(
            chain
                .and_then(|value| fetch(FastValue(value)))
                .zip(fetch(FastValue(10_000))),
        )
    );

    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert!(wake.take_notification());

    let result = run(future);

    assert_eq!(result.unwrap(), (expected, 10_000));
    assert_eq!(
        *context.fast_batches.borrow(),
        [HashMap::from([
            (FastValue(expected), expected),
            (FastValue(10_000), 10_000),
        ])]
    );
}

#[test]
fn requests_stay_batched_when_dependent_computations_yield() {
    let mut chain = Fetch::pure(0);
    for _ in 0..4096 {
        chain = chain.and_then(|value| Fetch::pure(value + 1));
    }

    assert_yield_preserves_batching(chain, 4096);
}

#[test]
fn requests_stay_batched_when_mapping_yields() {
    let mut chain = Fetch::pure(0);
    for _ in 0..4096 {
        chain = chain.map(|value| value + 1);
    }

    assert_yield_preserves_batching(chain, 4096);
}
