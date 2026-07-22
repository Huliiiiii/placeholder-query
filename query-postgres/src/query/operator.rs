#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    And,
    Or,
    Eq,
    In,
    Like,
}
