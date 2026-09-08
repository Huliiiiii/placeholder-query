pub(crate) mod ast;
mod dsl;
pub mod predicate;
mod render;
mod scope;
pub mod source;
pub mod template;
#[cfg(test)]
mod tests;

use std::{marker::PhantomData, sync::Arc};

use crate::query::{
    Projection,
    params::{InputScope, QueryInput},
    select::{
        ast::{SelectAst, SelectBody},
        source::Source,
        template::SelectTemplate,
    },
};

pub fn query<I: QueryInput + ?Sized, P>(
    build: impl FnOnce(I::Params) -> Select<P>,
) -> Select<P, I> {
    let input_scope = InputScope::new();
    let Select {
        body,
        projection,
        input_scope: _,
        _input: _,
    } = build(I::params(input_scope));

    Select {
        body,
        projection,
        input_scope: Some(input_scope),
        _input: PhantomData,
    }
}

pub fn from<T>(source: impl Into<Source<T>>) -> Select<T> {
    let Source {
        relation,
        projection,
    } = source.into();

    Select {
        body: SelectBody::new(relation),
        projection,
        input_scope: None,
        _input: PhantomData,
    }
}

#[derive_where::derive_where(Debug; Projection: std::fmt::Debug)]
pub struct Select<Projection, Input: ?Sized = ()> {
    pub(crate) body: SelectBody,
    pub(crate) projection: Projection,
    pub(crate) input_scope: Option<InputScope>,
    pub(crate) _input: PhantomData<fn(&Input)>,
}

impl<P: Projection, Input: ?Sized> Select<P, Input> {
    pub fn compile(self) -> SelectTemplate<P, Input> {
        let Self {
            body,
            projection,
            input_scope,
            _input: _,
        } = self;

        let ast = SelectAst {
            body,
            projection: projection.select_exprs(),
        };

        SelectTemplate {
            statement: Arc::new(render::render(&ast, input_scope)),
            projection: Arc::new(projection),
            _input: PhantomData,
        }
    }
}

pub struct BoundSelect<'a, Projection, Input: ?Sized> {
    pub select: SelectTemplate<Projection, Input>,
    pub params: &'a Input,
}
