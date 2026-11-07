use std::task::{Poll, Waker};

use crate::FetchError;

use super::value::Value;

#[derive(Clone, Copy)]
pub(super) struct ResultId(usize);

pub(super) struct ResultVar(ResultId);

impl ResultVar {
    pub(super) fn id(&self) -> ResultId {
        self.0
    }
}

enum ResultSlot<E> {
    Pending(Vec<Waker>),
    Completed(Result<Value, FetchError<E>>),
}

pub(super) struct ResultSlots<E> {
    slots: Vec<ResultSlot<E>>,
}

impl<E: Clone> ResultSlots<E> {
    pub(super) fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub(super) fn alloc(&mut self) -> ResultVar {
        let id = ResultId(self.slots.len());
        self.slots.push(ResultSlot::Pending(Vec::new()));
        ResultVar(id)
    }

    pub(super) fn poll<T: Clone + 'static>(
        &mut self,
        id: ResultId,
        waker: &Waker,
    ) -> Poll<Result<T, FetchError<E>>> {
        match &mut self.slots[id.0] {
            ResultSlot::Pending(waiters) => {
                if !waiters.iter().any(|waiter| waiter.will_wake(waker)) {
                    waiters.push(waker.clone());
                }
                Poll::Pending
            }
            ResultSlot::Completed(result) => Poll::Ready(match result {
                Ok(value) => Ok(value.downcast_ref::<T>().clone()),
                Err(error) => Err(error.clone()),
            }),
        }
    }

    pub(super) fn complete(
        &mut self,
        rvar: ResultVar,
        value: Result<Value, FetchError<E>>,
    ) -> Vec<Waker> {
        match std::mem::replace(&mut self.slots[rvar.0.0], ResultSlot::Completed(value)) {
            ResultSlot::Pending(waiters) => waiters,
            ResultSlot::Completed(_) => unreachable!("request should complete only once"),
        }
    }
}
