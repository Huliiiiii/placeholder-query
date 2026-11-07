use std::{collections::HashMap, future::Future, hash::Hash};

pub trait Request: Clone + Eq + Hash + Send + Sync {
    type Output;
}

pub trait Env: Sized + Sync {
    type Error: Clone + Send;
}

pub trait DataSource<R>: Env
where
    R: Request,
{
    /// Fetches one output for each request.
    ///
    /// `reqs` must be unique.
    fn fetch<'a>(
        &self,
        reqs: impl IntoIterator<Item = &'a R> + Send,
    ) -> impl Future<Output = Result<HashMap<R, R::Output>, Self::Error>> + Send
    where
        R: 'a;
}
