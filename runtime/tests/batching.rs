use std::{any::Any, collections::HashMap, sync::Mutex};

use futures_util::{future::try_join_all, join, try_join};
use ruxl::{DataSource, Env, Request};

#[derive(Default)]
struct Backend {
    batches: Mutex<Vec<Box<dyn Any + Send>>>,
}

impl Env for Backend {
    type Error = ();
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Number(i32);

impl Request for Number {
    type Output = i32;
}

impl DataSource<Number> for Backend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a Number>,
    ) -> Result<HashMap<Number, i32>, ()> {
        let mut reqs = reqs.into_iter().cloned().collect::<Vec<_>>();
        reqs.sort_unstable_by_key(|req| req.0);
        self.batches.lock().unwrap().push(Box::new(reqs.clone()));

        Ok(reqs.into_iter().map(|req| (req.clone(), req.0)).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Text(i32);

impl Request for Text {
    type Output = String;
}

impl DataSource<Text> for Backend {
    async fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a Text>,
    ) -> Result<HashMap<Text, String>, ()> {
        let mut reqs = reqs.into_iter().cloned().collect::<Vec<_>>();
        reqs.sort_unstable_by_key(|req| req.0);
        self.batches.lock().unwrap().push(Box::new(reqs.clone()));

        Ok(reqs
            .into_iter()
            .map(|req| (req.clone(), req.0.to_string()))
            .collect())
    }
}

#[tokio::test]
async fn deduplication_preserves_results_for_each_input() {
    let backend = Backend::default();
    let values = ruxl::run(&backend, async |scope| {
        ruxl::traverse([2, 1, 2, 1], |id| scope.fetch(Number(id))).await
    })
    .await
    .unwrap();

    assert_eq!(values, [2, 1, 2, 1]);
    let batches = backend.batches.into_inner().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(
        batches[0].downcast_ref::<Vec<Number>>().unwrap(),
        &[Number(1), Number(2)]
    );
}

#[tokio::test]
async fn different_request_types_are_fetched_separately() {
    let backend = Backend::default();
    let result = ruxl::run(&backend, async |scope| {
        try_join!(scope.fetch(Number(7)), scope.fetch(Text(7)))
    })
    .await
    .unwrap();

    assert_eq!(result, (7, String::from("7")));
    let batches = backend.batches.into_inner().unwrap();
    assert_eq!(batches.len(), 2);
    assert!(batches.iter().any(|batch| {
        batch
            .downcast_ref::<Vec<Number>>()
            .is_some_and(|reqs| reqs == &[Number(7)])
    }));
    assert!(batches.iter().any(|batch| {
        batch
            .downcast_ref::<Vec<Text>>()
            .is_some_and(|reqs| reqs == &[Text(7)])
    }));
}

#[tokio::test]
async fn cached_results_are_reused_across_dependent_computations() {
    let backend = Backend::default();
    let result = ruxl::run(&backend, async |scope| {
        scope.fetch(Number(7)).await?;
        scope.fetch(Number(7)).await
    })
    .await
    .unwrap();

    assert_eq!(result, 7);
    let batches = backend.batches.into_inner().unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(
        batches[0].downcast_ref::<Vec<Number>>().unwrap(),
        &[Number(7)]
    );
}

#[tokio::test]
async fn requests_from_parallel_branches_are_batched_together() {
    let backend = Backend::default();
    let result = ruxl::run(&backend, async |scope| {
        let count = scope.fetch(Number(2)).await?;
        try_join!(
            try_join_all((0..count).map(|id| scope.fetch(Number(id + 7)))),
            scope.fetch(Number(9)),
        )
    })
    .await
    .unwrap();

    assert_eq!(result, (vec![7, 8], 9));
    let batches = backend.batches.into_inner().unwrap();
    assert_eq!(batches.len(), 2);
    assert_eq!(
        batches[0].downcast_ref::<Vec<Number>>().unwrap(),
        &[Number(2)]
    );
    assert_eq!(
        batches[1].downcast_ref::<Vec<Number>>().unwrap(),
        &[Number(7), Number(8), Number(9)]
    );
}

#[tokio::test]
async fn concurrent_runs_keep_their_request_caches_separate() {
    let backend = Backend::default();
    let (left, right) = join!(
        ruxl::run(&backend, async |scope| scope.fetch(Number(7)).await),
        ruxl::run(&backend, async |scope| scope.fetch(Number(7)).await),
    );

    assert_eq!((left.unwrap(), right.unwrap()), (7, 7));
    let batches = backend.batches.into_inner().unwrap();
    assert_eq!(batches.len(), 2);
    for batch in batches {
        assert_eq!(batch.downcast_ref::<Vec<Number>>().unwrap(), &[Number(7)]);
    }
}
