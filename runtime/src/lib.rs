mod batch;
mod error;
mod fetch;

pub use batch::{DataSource, Env, Request};
pub use error::FetchError;
pub use fetch::{Fetch, fetch, traverse};
