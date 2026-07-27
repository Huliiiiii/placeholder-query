use crate::ident::{Ident, TableAlias};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ColumnRef {
    pub(crate) schema: Option<Ident>,
    pub(crate) table_alias: TableAlias,
    pub(crate) name: Ident,
}

impl ColumnRef {
    pub fn schema(&self) -> Option<&Ident> {
        self.schema.as_ref()
    }

    pub fn table_alias(&self) -> TableAlias {
        self.table_alias
    }

    pub fn name(&self) -> &Ident {
        &self.name
    }
}
