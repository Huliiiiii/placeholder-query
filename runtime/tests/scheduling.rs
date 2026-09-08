mod support;

use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    future::{Future, poll_fn},
    task::{Context, Poll, Waker},
};

use placeholder_query_runtime::{DataSource, Fetch, FetchEnv, Request, fetch, traverse};

use support::{TestWake, run_future, yield_once};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FastValue(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct SlowValue(i32);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct DependentValue(i32);

#[derive(Clone, Debug, PartialEq, Eq)]
enum SchedulingEvent {
    FastStarted(Vec<i32>),
    FastCompleted,
    SlowStarted(Vec<i32>),
    SlowCompleted,
    DependentStarted(Vec<i32>),
}

struct SchedulingContext {
    events: RefCell<Vec<SchedulingEvent>>,
    fast_ready: Cell<bool>,
    fast_waker: RefCell<Option<Waker>>,
    slow_ready: Cell<bool>,
    slow_waker: RefCell<Option<Waker>>,
}

impl SchedulingContext {
    fn new() -> Self {
        Self {
            events: RefCell::new(Vec::new()),
            fast_ready: Cell::new(false),
            fast_waker: RefCell::new(None),
            slow_ready: Cell::new(false),
            slow_waker: RefCell::new(None),
        }
    }

    fn events(&self) -> Vec<SchedulingEvent> {
        self.events.borrow().clone()
    }

    fn release_fast_batch(&self) {
        self.fast_ready.set(true);

        if let Some(waker) = self.fast_waker.borrow_mut().take() {
            waker.wake();
        }
    }

    fn release_slow_batch(&self) {
        self.slow_ready.set(true);

        if let Some(waker) = self.slow_waker.borrow_mut().take() {
            waker.wake();
        }
    }
}

impl FetchEnv for SchedulingContext {
    type Error = RequestError;
}

impl Request for FastValue {
    type Output = i32;
}

impl DataSource<FastValue> for SchedulingContext {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a FastValue>,
    ) -> impl Future<Output = Result<HashMap<FastValue, i32>, RequestError>> {
        let reqs = reqs.into_iter().collect::<Vec<_>>();

        let ids = reqs.iter().map(|req| req.0).collect::<Vec<_>>();
        self.events
            .borrow_mut()
            .push(SchedulingEvent::FastStarted(ids));
        let values = reqs
            .into_iter()
            .cloned()
            .map(|req| {
                let value = req.0;
                (req, value)
            })
            .collect();

        async move {
            poll_fn(|cx| {
                if self.fast_ready.get() {
                    Poll::Ready(())
                } else {
                    self.fast_waker.borrow_mut().replace(cx.waker().clone());

                    Poll::Pending
                }
            })
            .await;
            self.events
                .borrow_mut()
                .push(SchedulingEvent::FastCompleted);

            Ok(values)
        }
    }
}

impl Request for SlowValue {
    type Output = i32;
}

impl DataSource<SlowValue> for SchedulingContext {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a SlowValue>,
    ) -> impl Future<Output = Result<HashMap<SlowValue, i32>, RequestError>> {
        let reqs = reqs.into_iter().collect::<Vec<_>>();

        let ids = reqs.iter().map(|req| req.0).collect::<Vec<_>>();
        self.events
            .borrow_mut()
            .push(SchedulingEvent::SlowStarted(ids));
        let values = reqs
            .into_iter()
            .cloned()
            .map(|req| {
                let value = req.0;
                (req, value)
            })
            .collect();

        async move {
            poll_fn(|cx| {
                if self.slow_ready.get() {
                    Poll::Ready(())
                } else {
                    self.slow_waker.borrow_mut().replace(cx.waker().clone());

                    Poll::Pending
                }
            })
            .await;
            self.events
                .borrow_mut()
                .push(SchedulingEvent::SlowCompleted);

            Ok(values)
        }
    }
}

impl Request for DependentValue {
    type Output = i32;
}

impl DataSource<DependentValue> for SchedulingContext {
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a DependentValue>,
    ) -> impl Future<Output = Result<HashMap<DependentValue, i32>, RequestError>> {
        let reqs = reqs.into_iter().collect::<Vec<_>>();

        let ids = reqs.iter().map(|req| req.0).collect::<Vec<_>>();
        self.events
            .borrow_mut()
            .push(SchedulingEvent::DependentStarted(ids));
        let values = reqs
            .into_iter()
            .cloned()
            .map(|req| {
                let value = req.0;
                (req, value)
            })
            .collect();

        async move {
            yield_once().await;

            Ok(values)
        }
    }
}

#[test]
fn completed_batch_unlocks_a_dependent_batch_while_another_batch_is_in_flight() {
    let context = SchedulingContext::new();
    let fetch = traverse([1, 2, 1], |id| fetch(FastValue(id)))
        .and_then(|values| traverse(values, |value| fetch(DependentValue(value + 10))))
        .zip(fetch(SlowValue(99)));
    let wake = TestWake::new();
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(FetchEnv::run(&context, fetch));

    while wake.take_notification() {
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }

    let events = context.events();
    assert!(events.iter().any(|event| matches!(
        event,
        SchedulingEvent::FastStarted(ids)
            if ids.len() == 2 && ids.contains(&1) && ids.contains(&2)
    )));
    assert!(events.contains(&SchedulingEvent::SlowStarted(vec![99])));
    assert!(
        !events
            .iter()
            .any(|event| matches!(event, SchedulingEvent::DependentStarted(_)))
    );

    context.release_fast_batch();

    while wake.take_notification() {
        assert!(future.as_mut().poll(&mut cx).is_pending());
    }

    let events = context.events();
    assert!(events.iter().any(|event| matches!(
        event,
        SchedulingEvent::DependentStarted(ids)
            if ids.len() == 2 && ids.contains(&11) && ids.contains(&12)
    )));
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event, SchedulingEvent::DependentStarted(_)))
            .count(),
        1
    );
    assert!(!events.contains(&SchedulingEvent::SlowCompleted));

    context.release_slow_batch();

    let result = run_future(future, wake);

    assert_eq!(result.unwrap(), (vec![11, 12, 11], 99));
}

#[test]
fn yielding_a_runnable_chain_keeps_independent_requests_in_one_batch() {
    let context = SchedulingContext::new();
    context.release_fast_batch();

    let mut chain = Fetch::pure(0);
    for _ in 0..4096 {
        chain = chain.and_then(|value| Fetch::pure(value + 1));
    }

    let wake = TestWake::new();
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(FetchEnv::run(
        &context,
        chain
            .and_then(|value| fetch(FastValue(value)))
            .zip(fetch(FastValue(10_000))),
    ));

    assert!(wake.take_notification());
    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert!(context.events().is_empty());

    let result = run_future(future, wake);

    assert_eq!(result.unwrap(), (4096, 10_000));
    assert_eq!(
        context
            .events()
            .iter()
            .filter(|event| matches!(event, SchedulingEvent::FastStarted(_)))
            .count(),
        1
    );
    assert!(context.events().iter().any(|event| matches!(
        event,
        SchedulingEvent::FastStarted(ids)
            if ids.len() == 2 && ids.contains(&4096) && ids.contains(&10_000)
    )));
}
