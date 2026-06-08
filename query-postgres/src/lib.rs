use std::marker::PhantomData;

use placeholder_query_runtime::FetchBackend;

mod fetch;
pub mod query;

pub use query::select::{Pg, PgQueryCx, PgSelect, PgStatement};

mod utils;

#[derive(Clone, Copy, Debug)]
pub struct PgFetchBackend<Row, Error> {
    _marker: PhantomData<fn() -> (Row, Error)>,
}

impl<Row, Error> FetchBackend for PgFetchBackend<Row, Error> {
    type Request = PgStatement;
    type Row = Row;
    type Error = Error;
}
