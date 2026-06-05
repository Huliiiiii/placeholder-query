use std::borrow::Cow;

pub type Ident = Cow<'static, str>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub(crate) schema: Option<Ident>,
    pub(crate) table: Ident,
    pub(crate) name: Ident,
}
