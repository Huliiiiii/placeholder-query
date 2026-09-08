use std::ops::{Index, IndexMut};

use super::continuation::{Job, Value};

#[derive(Clone, Copy)]
pub(super) struct ResultId(usize);

#[derive_where::derive_where(Default)]
pub(super) struct ResultSlots<E> {
    slots: Vec<ResultSlot<E>>,
}

impl<E> ResultSlots<E> {
    pub(super) fn alloc(&mut self) -> ResultId {
        let id = ResultId(self.slots.len());
        self.slots.push(ResultSlot::Pending(Vec::new()));
        id
    }
}

impl<E> Index<ResultId> for ResultSlots<E> {
    type Output = ResultSlot<E>;

    fn index(&self, id: ResultId) -> &Self::Output {
        &self.slots[id.0]
    }
}

impl<E> IndexMut<ResultId> for ResultSlots<E> {
    fn index_mut(&mut self, id: ResultId) -> &mut Self::Output {
        &mut self.slots[id.0]
    }
}

pub(super) enum ResultSlot<E> {
    Pending(Vec<Job<E>>),
    Ready(Value),
    Taken,
}

impl<E> ResultSlot<E> {
    pub(super) fn is_ready(&self) -> bool {
        matches!(self, Self::Ready(_))
    }
}
