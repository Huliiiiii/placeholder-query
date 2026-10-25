use super::{continuation::Continuation, result::ResultId, state::FetchState, value::Value};

pub(super) enum Step<E> {
    Ready(Value),
    Blocked(ResultId, Resumption<E>),
}

struct Children<E>(Vec<Computation<E>>);

impl<E> Drop for Children<E> {
    fn drop(&mut self) {
        // Flatten the resumption tree to avoid stack overflows when dropping deeply nested resumptions.
        while let Some(mut child) = self.0.pop() {
            self.0.append(&mut child.resumption.children.0);
        }
    }
}

type Callback<E> = Box<dyn FnOnce(&mut FetchState<E>, Vec<Computation<E>>) -> Step<E>>;

pub(super) struct Resumption<E> {
    children: Children<E>,
    callback: Callback<E>,
}

impl<E> Resumption<E> {
    pub(super) fn with(
        children: Vec<Computation<E>>,
        callback: impl FnOnce(&mut FetchState<E>, Vec<Computation<E>>) -> Step<E> + 'static,
    ) -> Self {
        Self {
            children: Children(children),
            callback: Box::new(callback),
        }
    }

    pub(super) fn new(callback: impl FnOnce(&mut FetchState<E>) -> Step<E> + 'static) -> Self {
        Self::with(Vec::new(), move |state, _| callback(state))
    }

    pub(super) fn run(self, state: &mut FetchState<E>) -> Step<E> {
        let Self {
            mut children,
            callback,
        } = self;

        callback(state, std::mem::take(&mut children.0))
    }
}

pub(super) struct Computation<E> {
    pub(super) resumption: Resumption<E>,
    pub(super) conts: Vec<Continuation<E>>,
}
