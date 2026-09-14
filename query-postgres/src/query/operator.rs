#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    IsNotNull,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    And,
    Or,
    Eq,
    EqAny,
    EqAll,
    Gt,
    GtAny,
    GtAll,
    Gte,
    GteAny,
    GteAll,
    Like,
    LikeAny,
    LikeAll,
}
