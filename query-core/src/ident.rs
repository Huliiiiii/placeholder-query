use std::{borrow::Cow, fmt};

pub type Ident = Cow<'static, str>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TableAlias(pub u32);

impl fmt::Display for TableAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}
