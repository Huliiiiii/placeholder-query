pub trait QueryBackend {
    type BinaryOp;
    type UnaryOp;
    type Value;
}
