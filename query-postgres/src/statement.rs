use crate::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub struct PgStatement {
    pub sql: String,
    pub params: Vec<Value>,
}
