use crate::query::params::SqlType;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub(crate) sql: String,
    pub(crate) params: Vec<Param>,
}

impl Statement {
    pub fn sql(&self) -> &str {
        &self.sql
    }

    pub fn params(&self) -> &[Param] {
        &self.params
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Param {
    pub index: usize,
    pub ty: SqlType,
}
