use super::{
    computation::{Computation, Resumption, Step},
    result::ResultVar,
    state::FetchState,
    value::Value,
};

pub(super) enum Continuation<E> {
    Map(Box<dyn FnOnce(Value) -> Value>),
    Bind(Box<dyn FnOnce(Value) -> Computation<E>>),
}

impl<E> Continuation<E> {
    pub(super) fn map<A: 'static, C: 'static>(map: impl FnOnce(A) -> C + 'static) -> Self {
        Self::Map(Box::new(|value| Value::new(map(value.into_inner::<A>()))))
    }

    pub(super) fn bind<A: 'static>(then: impl FnOnce(A) -> Computation<E> + 'static) -> Self {
        Self::Bind(Box::new(|value| then(value.into_inner::<A>())))
    }
}

struct ContinuationStack<E>(Vec<Continuation<E>>);

impl<E> From<Vec<Continuation<E>>> for ContinuationStack<E> {
    fn from(mut conts: Vec<Continuation<E>>) -> Self {
        conts.reverse();

        Self(conts)
    }
}

impl<E> ContinuationStack<E> {
    fn pop(&mut self) -> Option<Continuation<E>> {
        self.0.pop()
    }

    fn prepend(&mut self, conts: Vec<Continuation<E>>) {
        self.0.extend(conts.into_iter().rev());
    }
}

pub(super) struct Job<E> {
    resumption: Resumption<E>,
    cont_stack: ContinuationStack<E>,
    rvar: ResultVar,
}

impl<E> Job<E> {
    pub(super) fn new(comp: Computation<E>, rvar: ResultVar) -> Self {
        Self {
            resumption: comp.resumption,
            cont_stack: comp.conts.into(),
            rvar,
        }
    }

    pub(super) fn run(self, state: &mut FetchState<E>, budget: &mut usize) {
        let Self {
            resumption,
            mut cont_stack,
            rvar,
        } = self;

        match resumption.run(state) {
            Step::Ready(mut value) => {
                let resumption = loop {
                    let Some(cont) = cont_stack.pop() else {
                        state.complete(rvar, value);

                        return;
                    };

                    match cont {
                        Continuation::Map(map) => {
                            value = map(value);

                            if *budget == 0 {
                                break Resumption::new(move |_| Step::Ready(value));
                            }

                            *budget -= 1;
                        }
                        Continuation::Bind(bind) => {
                            let comp = bind(value);
                            cont_stack.prepend(comp.conts);

                            break comp.resumption;
                        }
                    }
                };

                state.runnable.push(Self {
                    resumption,
                    cont_stack,
                    rvar,
                });
            }
            Step::Blocked(id, resumption) => state.results.wait(
                id,
                Self {
                    resumption,
                    cont_stack,
                    rvar,
                },
            ),
        }
    }
}
