use std::{
    any::{Any, TypeId, type_name},
    future::Future,
    pin::Pin,
};

use indexmap::{IndexMap, map::Entry as IndexEntry};

use crate::{
    FetchError,
    batch::{DataSource, Env, Request},
};

use super::{result::ResultVar, value::Value};

type CompletedBatch = Vec<(ResultVar, Value)>;

pub(super) type ExecuteBatchFuture<'a, E> =
    Pin<Box<dyn Future<Output = Result<CompletedBatch, FetchError<<E as Env>::Error>>> + 'a>>;

trait PendingBatch<E> {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_job<'a>(self: Box<Self>, context: &'a E) -> ExecuteBatchFuture<'a, E>
    where
        E: Env;
}

impl<E, R> PendingBatch<E> for Vec<(R, ResultVar)>
where
    E: DataSource<R>,
    R: Request + 'static,
    R::Output: 'static,
{
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn into_job<'a>(self: Box<Self>, context: &'a E) -> ExecuteBatchFuture<'a, E> {
        let batch = *self;

        Box::pin(async move {
            let mut outputs = context.fetch(batch.iter().map(|(req, _)| req)).await?;

            batch
                .into_iter()
                .map(|(req, rvar)| {
                    outputs
                        .remove(&req)
                        .map(|output| (rvar, Value::new(output)))
                        .ok_or_else(|| FetchError::MissingOutput {
                            req_type: type_name::<R>(),
                        })
                })
                .collect()
        })
    }
}

pub(super) struct RequestStore<E> {
    batches: IndexMap<TypeId, Box<dyn PendingBatch<E>>>,
}

impl<E> RequestStore<E> {
    pub(super) fn new() -> Self {
        Self {
            batches: IndexMap::new(),
        }
    }

    pub(super) fn insert<R>(&mut self, req: R, rvar: ResultVar)
    where
        E: DataSource<R>,
        R: Request + 'static,
        R::Output: 'static,
    {
        match self.batches.entry(TypeId::of::<R>()) {
            IndexEntry::Vacant(entry) => {
                entry.insert(Box::new(vec![(req, rvar)]));
            }
            IndexEntry::Occupied(entry) => {
                let batch = entry
                    .into_mut()
                    .as_any_mut()
                    .downcast_mut::<Vec<(R, ResultVar)>>()
                    .expect("request store batch type should match request type");

                batch.push((req, rvar));
            }
        }
    }

    pub(super) fn take_jobs<'a>(&mut self, context: &'a E) -> Vec<ExecuteBatchFuture<'a, E>>
    where
        E: Env,
    {
        std::mem::take(&mut self.batches)
            .into_values()
            .map(|batch| batch.into_job(context))
            .collect()
    }
}
