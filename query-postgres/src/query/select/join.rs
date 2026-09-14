use std::marker::PhantomData;

use crate::query::{Expr, Projection, projection::OptionalRow};

use super::{
    Select,
    ast::{self, FromClause, JoinKind, SelectBody},
    source::Source,
};

pub struct Join<L, R, I: ?Sized = ()> {
    inputs: JoinInputs<L, R, I>,
}

impl<L: Projection, R: Projection, I: ?Sized> Join<L, R, I> {
    pub fn on(self, predicate: impl FnOnce(&L, &R) -> Expr<bool>) -> Select<(L, R), I> {
        self.inputs
            .finish_on(predicate, |left, right| (left, right))
    }
}

pub struct LeftJoin<L, R, I: ?Sized = ()> {
    inputs: JoinInputs<L, R, I>,
    tag: Expr<bool>,
}

impl<L: Projection, R: Projection, I: ?Sized> LeftJoin<L, R, I> {
    pub fn on(
        self,
        predicate: impl FnOnce(&L, &R) -> Expr<bool>,
    ) -> Select<(L, OptionalRow<R>), I> {
        self.inputs.finish_on(predicate, |left, right| {
            (left, OptionalRow::new(self.tag, right))
        })
    }

    pub fn on_true(self) -> Select<(L, OptionalRow<R>), I> {
        self.on(|_, _| Expr::true_literal())
    }
}

pub struct RightJoin<L, R, I: ?Sized = ()> {
    inputs: JoinInputs<(Expr<bool>, L), R, I>,
}

impl<L: Projection, R: Projection, I: ?Sized> RightJoin<L, R, I> {
    pub fn on(
        self,
        predicate: impl FnOnce(&L, &R) -> Expr<bool>,
    ) -> Select<(OptionalRow<L>, R), I> {
        self.inputs.finish_on(
            |(_, left), right| predicate(left, right),
            |(tag, left), right| (OptionalRow::new(tag, left), right),
        )
    }
}

pub struct FullJoin<L, R, I: ?Sized = ()> {
    inputs: JoinInputs<(Expr<bool>, L), R, I>,
    tag: Expr<bool>,
}

impl<L: Projection, R: Projection, I: ?Sized> FullJoin<L, R, I> {
    pub fn on(
        self,
        predicate: impl FnOnce(&L, &R) -> Expr<bool>,
    ) -> Select<(OptionalRow<L>, OptionalRow<R>), I> {
        self.inputs.finish_on(
            |(_, left), right| predicate(left, right),
            |(tag, left), right| {
                (
                    OptionalRow::new(tag, left),
                    OptionalRow::new(self.tag, right),
                )
            },
        )
    }
}

struct JoinInputs<L, R, I: ?Sized> {
    left: Select<L, I>,
    right: Source<R>,
    kind: JoinKind,
    lateral: bool,
}

impl<L: Projection, R: Projection, I: ?Sized> JoinInputs<L, R, I> {
    fn lateral(mut self) -> Self {
        self.lateral = true;
        self
    }

    fn with_right_maybe_tag(self) -> (Self, Expr<bool>) {
        let (right, tag) = self.right.with_maybe_tag();
        (Self { right, ..self }, tag)
    }

    fn finish_on<O: Projection>(
        self,
        predicate: impl FnOnce(&L, &R) -> Expr<bool>,
        output: impl FnOnce(L, R) -> O,
    ) -> Select<O, I> {
        let Select {
            mut body,
            projection: left,
            input_scope,
            _input: _,
        } = self.left;
        let Source {
            relation,
            projection: right,
        } = self.right;

        let on = predicate(&left, &right).erase();
        body.from.joins.push(ast::Join {
            kind: self.kind,
            lateral: self.lateral,
            relation,
            on: Some(on),
        });

        Select {
            body,
            projection: output(left, right),
            input_scope,
            _input: PhantomData,
        }
    }

    fn finish<O: Projection>(self, output: impl FnOnce(L, R) -> O) -> Select<O, I> {
        let Select {
            mut body,
            projection: left,
            input_scope,
            _input: _,
        } = self.left;
        let Source {
            relation,
            projection: right,
        } = self.right;

        body.from.joins.push(ast::Join {
            kind: self.kind,
            lateral: self.lateral,
            relation,
            on: None,
        });

        Select {
            body,
            projection: output(left, right),
            input_scope,
            _input: PhantomData,
        }
    }
}

impl<L: Projection, I: ?Sized> Select<L, I> {
    pub fn join<R: Projection>(self, source: impl Into<Source<R>>) -> Join<L, R, I> {
        Join {
            inputs: self.prepare_join(JoinKind::Inner, move |_| source.into()),
        }
    }

    pub fn left_join<R: Projection>(self, source: impl Into<Source<R>>) -> LeftJoin<L, R, I> {
        let (inputs, tag) = self
            .prepare_join(JoinKind::Left, move |_| source.into())
            .with_right_maybe_tag();
        LeftJoin { inputs, tag }
    }

    pub fn right_join<R: Projection>(self, source: impl Into<Source<R>>) -> RightJoin<L, R, I> {
        let inputs = self
            .map(|left| (Expr::true_literal(), left))
            .project_join_input()
            .prepare_join(JoinKind::Right, move |_| source.into());
        RightJoin { inputs }
    }

    pub fn full_join<R: Projection>(self, source: impl Into<Source<R>>) -> FullJoin<L, R, I> {
        let (inputs, tag) = self
            .map(|left| (Expr::true_literal(), left))
            .project_join_input()
            .prepare_join(JoinKind::Full, move |_| source.into())
            .with_right_maybe_tag();
        FullJoin { inputs, tag }
    }

    pub fn cross_join<R: Projection>(self, source: impl Into<Source<R>>) -> Select<(L, R), I> {
        self.prepare_join(JoinKind::Cross, move |_| source.into())
            .finish(|left, right| (left, right))
    }

    pub fn join_lateral<R: Projection>(self, build: impl FnOnce(&L) -> Select<R>) -> Join<L, R, I> {
        Join {
            inputs: self
                .prepare_join(JoinKind::Inner, |left| Source::from(build(left)))
                .lateral(),
        }
    }

    pub fn left_join_lateral<R: Projection>(
        self,
        build: impl FnOnce(&L) -> Select<R>,
    ) -> LeftJoin<L, R, I> {
        let (inputs, tag) = self
            .prepare_join(JoinKind::Left, |left| Source::from(build(left)))
            .lateral()
            .with_right_maybe_tag();
        LeftJoin { inputs, tag }
    }

    pub fn cross_join_lateral<R: Projection>(
        self,
        build: impl FnOnce(&L) -> Select<R>,
    ) -> Select<(L, R), I> {
        self.prepare_join(JoinKind::Cross, |left| Source::from(build(left)))
            .lateral()
            .finish(|left, right| (left, right))
    }

    fn prepare_join<R: Projection>(
        self,
        kind: JoinKind,
        build: impl FnOnce(&L) -> Source<R>,
    ) -> JoinInputs<L, R, I> {
        let right = build(&self.projection);

        JoinInputs {
            left: self,
            right,
            kind,
            lateral: false,
        }
    }

    fn project_join_input(self) -> Self {
        let Source {
            relation,
            projection,
        } = Source::project(self.body.from, self.projection);

        Select {
            body: SelectBody {
                from: FromClause::new(relation),
                ..self.body
            },
            projection,
            ..self
        }
    }
}
