use std::borrow::Cow;

pub type Ident = Cow<'static, str>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Column {
    pub(crate) schema: Option<Ident>,
    pub(crate) table: Ident,
    pub(crate) name: Ident,
}

impl Column {
    pub fn schema(&self) -> Option<&Ident> {
        self.schema.as_ref()
    }

    pub fn table(&self) -> &Ident {
        &self.table
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
