mod batch;
mod error;
mod fetch;

pub use batch::{DataSource, FetchEnv, Request};
pub use error::FetchError;
pub use fetch::{Fetch, fetch, traverse};
