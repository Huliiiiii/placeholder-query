use placeholder_query_core::backend::QueryBackend;

use crate::{
    query::{BinaryOp, UnaryOp},
    value::Value,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pg;

impl QueryBackend for Pg {
    type BinaryOp = BinaryOp;
    type UnaryOp = UnaryOp;
    type Value = Value;
}
