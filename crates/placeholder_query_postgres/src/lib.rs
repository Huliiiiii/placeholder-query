use std::marker::PhantomData;

use placeholder_query::QueryBuilder;

pub mod query;

pub use query::select::{PgQuery, PgQueryBuilder, PgQueryCx, PgSelect};

mod utils;

#[derive(Clone, Copy, Debug)]
pub struct PgBackend<Row, Error> {
    _marker: PhantomData<fn() -> (Row, Error)>,
}

impl<Row, Error> PgBackend<Row, Error> {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }
}

impl<Row, Error> QueryBuilder for PgBackend<Row, Error> {
    type Plan = query::select::PgQueryPlan;
    type Query = PgQuery;
    type Row = Row;
    type Error = Error;

    fn compile(&self, plan: &Self::Plan) -> Self::Query {
        PgQueryBuilder.build(plan)
    }
}
