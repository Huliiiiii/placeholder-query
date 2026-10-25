mod support;

use std::collections::HashMap;

use placeholder_query_runtime::{DataSource, Env, FetchError, Request, fetch, traverse};

use support::executor::run;

#[derive(Debug)]
struct RequestError;

struct ErrorContext;

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

#[test]
fn missing_batch_output_returns_runtime_error() {
    let result = run(ErrorContext.run(traverse([10, 20], |id| fetch(MissingOutputRequest(id)))));

    let Err(FetchError::MissingOutput { req_type }) = result else {
        panic!("missing batch output should return a runtime error");
    };
    assert_eq!(req_type, std::any::type_name::<MissingOutputRequest>());
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct FailingRequest;

impl Request for FailingRequest {
    type Output = ();
}

impl DataSource<FailingRequest> for ErrorContext {
    async fn fetch<'a>(
        &self,
        _: impl IntoIterator<Item = &'a FailingRequest>,
    ) -> Result<HashMap<FailingRequest, ()>, RequestError> {
        Err(RequestError)
    }
}

#[test]
fn datasource_errors_are_returned_to_the_caller() {
    let result = run(ErrorContext.run(fetch(FailingRequest)));

    assert!(matches!(result, Err(FetchError::Executor(RequestError))));
}
