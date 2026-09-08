use std::{marker::PhantomData, sync::Arc};

use crate::statement::Statement;

use super::BoundSelect;

pub struct SelectTemplate<Projection, Input: ?Sized = ()> {
    pub(crate) statement: Arc<Statement>,
    pub(crate) projection: Arc<Projection>,
    pub(crate) _input: PhantomData<fn(&Input)>,
}

impl<Projection, Input: ?Sized> Clone for SelectTemplate<Projection, Input> {
    fn clone(&self) -> Self {
        Self {
            statement: self.statement.clone(),
            projection: self.projection.clone(),
            _input: PhantomData,
        }
    }
}

impl<Projection, Input: ?Sized> SelectTemplate<Projection, Input> {
    pub fn statement(&self) -> &Arc<Statement> {
        &self.statement
    }

    pub fn projection(&self) -> &Projection {
        &self.projection
    }

    pub fn bind<'a>(&self, params: &'a Input) -> BoundSelect<'a, Projection, Input> {
        BoundSelect {
            select: self.clone(),
            params,
        }
    }
}
