use std::borrow::Cow;

pub type Ident = Cow<'static, str>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParamId {
    pub scope: u64,
    pub index: u16,
}

impl ParamId {
    pub fn new(scope: u64, index: u16) -> Self {
        Self { scope, index }
    }
}
