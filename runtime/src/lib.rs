mod batch;
mod driver;
mod error;

use std::future::Future;

use futures_util::future::try_join_all;

pub use batch::{DataSource, Env, Request};
pub use driver::{Scope, run};
pub use error::FetchError;

pub fn traverse<T, U, E, F>(
    items: impl IntoIterator<Item = T>,
    f: impl FnMut(T) -> F,
) -> impl Future<Output = Result<Vec<U>, E>>
where
    F: Future<Output = Result<U, E>>,
{
    try_join_all(items.into_iter().map(f))
}
