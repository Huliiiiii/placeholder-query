use std::{
    cell::Cell,
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

use futures_util::{future::join_all, join};
use ruxl::{DataSource, Env, FetchError, Request};

// Cell keeps this error Send without making it Sync.
#[derive(Clone, Debug)]
struct RequestError(Cell<usize>);

#[derive(Default)]
struct ErrorContext {
    calls: AtomicUsize,
}

impl Env for ErrorContext {
    type Error = RequestError;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct MissingOutputRequest(usize);

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

#[tokio::test]
async fn missing_output_only_fails_its_request() {
    let result = ruxl::run(&ErrorContext::default(), async |scope| {
        join_all(
            [10, 20]
                .into_iter()
                .map(|id| scope.fetch(MissingOutputRequest(id))),
        )
        .await
    })
    .await;

    assert_eq!(result[0].as_ref().unwrap(), &10);
    let Err(FetchError::MissingOutput { req_type }) = &result[1] else {
        panic!("missing batch output should return a runtime error");
    };
    assert_eq!(*req_type, std::any::type_name::<MissingOutputRequest>());
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FailingRequest(usize);

impl Request for FailingRequest {
    type Output = ();
}

impl DataSource<FailingRequest> for ErrorContext {
    async fn fetch<'a>(
        &self,
        _: impl IntoIterator<Item = &'a FailingRequest>,
    ) -> Result<HashMap<FailingRequest, ()>, RequestError> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Err(RequestError(Cell::new(7)))
    }
}

#[tokio::test]
async fn failed_requests_are_cached_without_stopping_other_requests() {
    let ((first, second, repeated, success), calls) = tokio::spawn(async {
        let context = ErrorContext::default();
        let results = ruxl::run(&context, async |scope| {
            let (first, second, success) = join!(
                scope.fetch(FailingRequest(1)),
                scope.fetch(FailingRequest(2)),
                scope.fetch(MissingOutputRequest(10)),
            );
            let repeated = scope.fetch(FailingRequest(1)).await;
            (first, second, repeated, success)
        })
        .await;
        (results, context.calls.load(Ordering::Relaxed))
    })
    .await
    .unwrap();

    for result in [first, second, repeated] {
        let Err(FetchError::Executor(error)) = result else {
            panic!("failed request should return the data source error");
        };
        assert_eq!(error.0.get(), 7);
    }
    assert_eq!(calls, 1);
    assert_eq!(success.unwrap(), 10);
}
