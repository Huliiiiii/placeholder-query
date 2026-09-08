use std::{
    future::Future,
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};

pub struct TestWake {
    notified: AtomicBool,
    deadline: Instant,
}

impl TestWake {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            notified: AtomicBool::new(true),
            deadline: Instant::now() + Duration::from_secs(10),
        })
    }

    pub fn take_notification(&self) -> bool {
        assert!(
            Instant::now() < self.deadline,
            "fetch exceeded the 10-second test deadline"
        );
        self.notified.swap(false, Ordering::Acquire)
    }
}

impl Wake for TestWake {
    fn wake(self: Arc<Self>) {
        self.notified.store(true, Ordering::Release);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.notified.store(true, Ordering::Release);
    }
}

pub fn run_future<F: Future>(future: F, wake: Arc<TestWake>) -> F::Output {
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);

    // All wakeups happen on this thread, so no notification means the fetch has stalled.
    loop {
        assert!(wake.take_notification(), "fetch stalled without a wakeup");
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => {}
        }
    }
}

pub fn yield_once() -> YieldOnce {
    YieldOnce(false)
}

pub struct YieldOnce(bool);

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            cx.waker().wake_by_ref();

            Poll::Pending
        }
    }
}
