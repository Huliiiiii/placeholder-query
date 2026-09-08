use std::{
    any::{Any, TypeId, type_name},
    future::Future,
    pin::Pin,
};

use indexmap::{IndexMap, map::Entry};

use crate::FetchError;
use crate::batch::{DataSource, FetchEnv, Request};

use super::{FetchState, result::ResultId};

pub(super) type CompleteBatchFn<E> = Box<dyn FnOnce(&mut FetchState<E>)>;

pub(super) type ExecuteBatchFuture<'a, E> = Pin<
    Box<dyn Future<Output = Result<CompleteBatchFn<E>, FetchError<<E as FetchEnv>::Error>>> + 'a>,
>;

#[derive_where::derive_where(Default)]
pub(super) struct RequestStore<E> {
    batches: IndexMap<TypeId, Box<dyn PendingBatch<E>>>,
}

impl<E> RequestStore<E> {
    pub(super) fn insert<R>(&mut self, req: R, id: ResultId)
    where
        E: DataSource<R>,
        R: Request + 'static,
        R::Output: 'static,
    {
        match self.batches.entry(TypeId::of::<R>()) {
            Entry::Vacant(entry) => {
                entry.insert(Box::new(vec![(req, id)]));
            }
            Entry::Occupied(entry) => {
                let batch = entry
                    .into_mut()
                    .as_any_mut()
                    .downcast_mut::<Vec<(R, ResultId)>>()
                    .expect("request store batch type should match request type");

                batch.push((req, id));
            }
        }
    }

    pub(super) fn take_jobs<'a>(&mut self, context: &'a E) -> Vec<ExecuteBatchFuture<'a, E>>
    where
        E: FetchEnv,
    {
        std::mem::take(&mut self.batches)
            .into_values()
            .map(|batch| batch.into_job(context))
            .collect()
    }
}

trait PendingBatch<E> {
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn into_job<'a>(self: Box<Self>, context: &'a E) -> ExecuteBatchFuture<'a, E>
    where
        E: FetchEnv;
}

impl<E, R> PendingBatch<E> for Vec<(R, ResultId)>
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
        Box::pin(async {
            let mut outputs = context.fetch(batch.iter().map(|(req, _)| req)).await?;

            let completed = batch
                .into_iter()
                .map(|(req, id)| {
                    outputs
                        .remove(&req)
                        .map(|output| (id, output))
                        .ok_or_else(|| FetchError::MissingOutput {
                            req_type: type_name::<R>(),
                        })
                })
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Box::new(move |state: &mut FetchState<E>| {
                for (id, output) in completed {
                    state.complete(id, Box::new(output));
                }
            }) as CompleteBatchFn<E>)
        })
    }
}
