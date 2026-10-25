use std::{
    future::Future,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};

pub struct TestWake {
    notified: AtomicBool,
}

impl TestWake {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            notified: AtomicBool::new(false),
        })
    }

    pub fn take_notification(&self) -> bool {
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

pub fn run<T>(future: impl Future<Output = T>) -> T {
    let wake = TestWake::new();
    let waker = Waker::from(wake.clone());
    let mut cx = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    let deadline = Instant::now() + Duration::from_secs(10);

    loop {
        assert!(Instant::now() < deadline, "test future exceeded 10 seconds");

        match future.as_mut().poll(&mut cx) {
            Poll::Ready(result) => return result,
            Poll::Pending => {
                assert!(
                    wake.take_notification(),
                    "test future stalled without a wakeup"
                );
            }
        }
    }
}
