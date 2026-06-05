use super::{
    plan::{PgQuery, PgQueryPlan},
    render::render_query,
};

#[derive(Clone, Copy, Debug, Default)]
pub struct PgQueryBuilder;

impl PgQueryBuilder {
    pub fn build(&self, query: &PgQueryPlan) -> PgQuery {
        render_query(query)
    }
}
