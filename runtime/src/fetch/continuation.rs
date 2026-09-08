use std::{any::Any, vec};

use super::{FetchState, result::ResultId};

pub(super) type Value = Box<dyn Any>;
pub(super) type StepFn<E> = Box<dyn for<'a> FnOnce(&'a mut FetchState<E>) -> Step<'a, E>>;
pub(super) type Continuation<E> = Box<dyn FnOnce(Value) -> Computation<E>>;

pub(super) struct Computation<E> {
    pub(super) step: StepFn<E>,
    pub(super) conts: Vec<Continuation<E>>,
}

pub(super) enum Step<'a, E> {
    Ready(Value),
    Blocked(&'a mut Vec<Job<E>>, StepFn<E>),
}

impl<E> Step<'_, E> {
    pub(super) fn ready(value: impl Any) -> Self {
        Self::Ready(Box::new(value))
    }
}

pub(super) struct Job<E> {
    step: StepFn<E>,
    cont_stack: Vec<vec::IntoIter<Continuation<E>>>,
    result: ResultId,
}

impl<E> Job<E> {
    pub(super) fn new(comp: Computation<E>, result: ResultId) -> Self {
        Self {
            step: comp.step,
            cont_stack: vec![comp.conts.into_iter()],
            result,
        }
    }

    pub(super) fn run(self, state: &mut FetchState<E>) {
        let Self {
            step,
            mut cont_stack,
            result,
        } = self;

        match step(state) {
            Step::Ready(value) => {
                let cont = loop {
                    let Some(conts) = cont_stack.last_mut() else {
                        state.complete(result, value);
                        return;
                    };

                    if let Some(cont) = conts.next() {
                        break cont;
                    }

                    cont_stack.pop();
                };

                let next = cont(value);

                // Keep inner binds ahead of the remaining outer chain without copying it.
                if !next.conts.is_empty() {
                    cont_stack.push(next.conts.into_iter());
                }

                state.runnable.prepend([Self {
                    step: next.step,
                    cont_stack,
                    result,
                }]);
            }
            Step::Blocked(waiters, resume) => {
                waiters.push(Self {
                    step: resume,
                    cont_stack,
                    result,
                });
            }
        }
    }
}
