use std::{collections::HashMap, future::Future, hash::Hash};

use crate::{Fetch, FetchError};

pub trait Request: Eq + Hash {
    type Output;
}

pub trait FetchEnv: Sized {
    type Error;

    fn run<A>(
        &self,
        computation: impl Into<Fetch<Self, A>>,
    ) -> impl Future<Output = Result<A, FetchError<Self::Error>>>
    where
        A: 'static,
    {
        computation.into().run_with(self)
    }
}

pub trait DataSource<R>: FetchEnv
where
    R: Request,
{
    /// Fetches one output for each request.
    ///
    /// `reqs` must be unique.
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a R>,
    ) -> impl Future<Output = Result<HashMap<R, R::Output>, Self::Error>>
    where
        R: 'a;
}
