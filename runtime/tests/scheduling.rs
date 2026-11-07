use std::{
    collections::HashMap,
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Wake, Waker},
    time::Duration,
};

use futures_util::{StreamExt, future::try_join_all, join, stream::FuturesUnordered};
use ruxl::{DataSource, Env, Request};
use tokio::{
    sync::{Notify, watch},
    time::timeout,
};

#[derive(Default)]
struct TestWake {
    notified: AtomicBool,
}

impl TestWake {
    fn take_notification(&self) -> bool {
        self.notified.swap(false, Ordering::Acquire)
    }
}

impl Wake for TestWake {
    fn wake(self: Arc<Self>) {
        self.wake_by_ref();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.notified.store(true, Ordering::Release);
    }
}

#[derive(Default)]
struct SchedulingContext {
    fast_batches: watch::Sender<Vec<Vec<i32>>>,
    slow_batches: watch::Sender<Vec<Vec<i32>>>,
    fast_ready: watch::Sender<bool>,
    slow_ready: watch::Sender<bool>,
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
        let mut ids = reqs.into_iter().map(|req| req.0).collect::<Vec<_>>();
        ids.sort_unstable();
        self.fast_batches
            .send_modify(|batches| batches.push(ids.clone()));
        self.fast_ready
            .subscribe()
            .wait_for(|ready| *ready)
            .await
            .unwrap();

        Ok(ids.into_iter().map(|id| (FastValue(id), id)).collect())
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
        let mut ids = reqs.into_iter().map(|req| req.0).collect::<Vec<_>>();
        ids.sort_unstable();
        self.slow_batches
            .send_modify(|batches| batches.push(ids.clone()));
        self.slow_ready
            .subscribe()
            .wait_for(|ready| *ready)
            .await
            .unwrap();

        Ok(ids.into_iter().map(|id| (SlowValue(id), id)).collect())
    }
}

#[tokio::test(start_paused = true)]
async fn dependent_requests_do_not_wait_for_unrelated_batches() {
    let context = SchedulingContext::default();
    let computation = ruxl::run(&context, async |scope| {
        join!(
            async {
                let values =
                    try_join_all([1, 2, 1].into_iter().map(|id| scope.fetch(FastValue(id))))
                        .await?;
                try_join_all(
                    values
                        .into_iter()
                        .map(|value| scope.fetch(FastValue(value + 10))),
                )
                .await
            },
            scope.fetch(SlowValue(99))
        )
    });
    let release_batches = async {
        let mut fast_batches = context.fast_batches.subscribe();
        let mut slow_batches = context.slow_batches.subscribe();
        fast_batches
            .wait_for(|batches| !batches.is_empty())
            .await
            .unwrap();
        slow_batches
            .wait_for(|batches| !batches.is_empty())
            .await
            .unwrap();
        assert_eq!(*fast_batches.borrow(), [vec![1, 2]]);
        assert_eq!(*slow_batches.borrow(), [vec![99]]);

        context.fast_ready.send_replace(true);
        fast_batches
            .wait_for(|batches| batches.len() == 2)
            .await
            .unwrap();
        assert_eq!(*fast_batches.borrow(), [vec![1, 2], vec![11, 12]]);
        assert_eq!(*slow_batches.borrow(), [vec![99]]);
        context.slow_ready.send_replace(true);
    };
    let (result, ()) = timeout(Duration::from_secs(1), async {
        join!(computation, release_batches)
    })
    .await
    .expect("dependent requests stalled behind an unrelated batch");

    assert_eq!(result.0.unwrap(), [11, 12, 11]);
    assert_eq!(result.1.unwrap(), 99);
}

#[tokio::test(start_paused = true)]
async fn duplicate_requests_wake_all_parallel_consumers() {
    let context = SchedulingContext::default();
    let wake = Arc::new(TestWake::default());
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(ruxl::run(&context, async |scope| {
        (0..3)
            .map(|_| scope.fetch(FastValue(7)))
            .collect::<FuturesUnordered<_>>()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
    }));

    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert_eq!(*context.fast_batches.borrow(), [vec![7]]);
    wake.take_notification();
    context.fast_ready.send_replace(true);
    assert!(wake.take_notification());
    let values = timeout(Duration::from_secs(1), future)
        .await
        .expect("parallel consumers were not all woken")
        .unwrap();
    assert_eq!(values, [7, 7, 7]);
}

#[tokio::test]
async fn computation_can_wait_for_an_external_event_before_requesting_data() {
    let context = SchedulingContext::default();
    context.fast_ready.send_replace(true);
    let gate = Notify::new();
    let wake = Arc::new(TestWake::default());
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(ruxl::run(&context, async |scope| {
        gate.notified().await;
        scope.fetch(FastValue(7)).await
    }));

    assert!(future.as_mut().poll(&mut cx).is_pending());
    assert!(context.fast_batches.borrow().is_empty());
    wake.take_notification();
    gate.notify_one();
    assert!(wake.take_notification());
    assert_eq!(future.await.unwrap(), 7);
}
