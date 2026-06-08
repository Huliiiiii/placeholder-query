use super::{
    plan::{PgSelectPlan, PgStatement},
    render::render_select_plan,
};

#[derive(Clone, Copy, Debug)]
pub struct Pg;

impl Pg {
    pub fn build(&self, plan: &PgSelectPlan) -> PgStatement {
        render_select_plan(plan)
    }
}
