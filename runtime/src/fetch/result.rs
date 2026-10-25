use super::{continuation::Job, value::Value};

#[derive(Clone, Copy)]
pub(super) struct ResultId(usize);

pub(super) struct ResultVar(ResultId);

impl ResultVar {
    pub(super) fn id(&self) -> ResultId {
        self.0
    }
}

enum ResultSlot<E> {
    Pending(Vec<Job<E>>),
    Completed(Option<Value>),
}

pub(super) struct ResultSlots<E> {
    slots: Vec<ResultSlot<E>>,
}

impl<E> ResultSlots<E> {
    pub(super) fn new() -> Self {
        Self { slots: Vec::new() }
    }

    pub(super) fn alloc(&mut self) -> ResultVar {
        let id = ResultId(self.slots.len());

        self.slots.push(ResultSlot::Pending(Vec::new()));

        ResultVar(id)
    }

    pub(super) fn wait(&mut self, id: ResultId, job: Job<E>) {
        match &mut self.slots[id.0] {
            ResultSlot::Pending(waiters) => {
                if waiters.len() == waiters.capacity() {
                    waiters.reserve_exact(waiters.capacity().max(1));
                }

                waiters.push(job);
            }
            ResultSlot::Completed(_) => {
                unreachable!("a blocked computation must wait for a pending result")
            }
        }
    }

    pub(super) fn complete(&mut self, rvar: ResultVar, value: Value) -> Vec<Job<E>> {
        let id = rvar.id();
        let previous = std::mem::replace(&mut self.slots[id.0], ResultSlot::Completed(Some(value)));

        match previous {
            ResultSlot::Pending(waiters) => waiters,
            ResultSlot::Completed(_) => {
                unreachable!("only pending results can be completed")
            }
        }
    }

    pub(super) fn get(&self, id: ResultId) -> Option<&Value> {
        match &self.slots[id.0] {
            ResultSlot::Completed(value) => value.as_ref(),
            ResultSlot::Pending(_) => None,
        }
    }

    pub(super) fn take(&mut self, id: ResultId) -> Option<Value> {
        match &mut self.slots[id.0] {
            ResultSlot::Completed(value) => value.take(),
            ResultSlot::Pending(_) => None,
        }
    }
}
