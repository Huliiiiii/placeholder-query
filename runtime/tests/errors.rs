mod support;

use std::{cell::Cell, collections::HashMap, future::pending, rc::Rc};

use placeholder_query_runtime::{DataSource, FetchEnv, FetchError, Request, fetch, traverse};

use support::{TestWake, run_future, yield_once};

#[derive(Clone, Debug, PartialEq, Eq)]
struct RequestError;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MissingOutputRequest(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct FailingRequest;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct PendingRequest;

struct DropSignal(Rc<Cell<bool>>);

impl Drop for DropSignal {
    fn drop(&mut self) {
        self.0.set(true);
    }
}

#[derive(Default)]
struct ErrorContext {
    pending_started: Cell<bool>,
    pending_dropped: Rc<Cell<bool>>,
}

impl FetchEnv for ErrorContext {
    type Error = RequestError;
}

impl Request for MissingOutputRequest {
    type Output = usize;
}

impl DataSource<MissingOutputRequest> for ErrorContext {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a MissingOutputRequest>,
    ) -> Result<HashMap<MissingOutputRequest, usize>, RequestError> {
        Ok(reqs.into_iter().take(1).map(|req| (*req, req.0)).collect())
    }
}

impl Request for FailingRequest {
    type Output = ();
}

impl DataSource<FailingRequest> for ErrorContext {
    async fn fetch<'a>(
        &self,
        _: impl IntoIterator<Item = &'a FailingRequest>,
    ) -> Result<HashMap<FailingRequest, ()>, RequestError> {
        yield_once().await;

        Err(RequestError)
    }
}

impl Request for PendingRequest {
    type Output = ();
}

impl DataSource<PendingRequest> for ErrorContext {
    fn fetch<'a>(
        &self,
        _: impl IntoIterator<Item = &'a PendingRequest>,
    ) -> impl Future<Output = Result<HashMap<PendingRequest, ()>, RequestError>> {
        async move {
            self.pending_started.set(true);
            let _drop_signal = DropSignal(self.pending_dropped.clone());

            pending().await
        }
    }
}

#[test]
fn missing_batch_output_returns_runtime_error() {
    let context = ErrorContext::default();
    let result = run_future(
        FetchEnv::run(
            &context,
            traverse([10, 20], |id| fetch(MissingOutputRequest(id))),
        ),
        TestWake::new(),
    );

    let Err(FetchError::MissingOutput { req_type }) = result else {
        panic!("missing batch output should return a runtime error");
    };
    assert_eq!(req_type, std::any::type_name::<MissingOutputRequest>());
}

#[test]
fn request_error_cancels_other_batches_and_stops_the_run() {
    let context = ErrorContext::default();
    let result = run_future(
        FetchEnv::run(&context, fetch(FailingRequest).zip(fetch(PendingRequest))),
        TestWake::new(),
    );

    assert!(matches!(result, Err(FetchError::Executor(RequestError))));
    assert!(context.pending_started.get());
    assert!(context.pending_dropped.get());
}
