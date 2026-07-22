use crate::ident::Ident;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnRef {
    pub(crate) schema: Option<Ident>,
    pub(crate) table_alias: Ident,
    pub(crate) name: Ident,
}

impl ColumnRef {
    pub fn schema(&self) -> Option<&Ident> {
        self.schema.as_ref()
    }

    pub fn table_alias(&self) -> &Ident {
        &self.table_alias
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
