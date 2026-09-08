use placeholder_query_core::types::Ident;

#[derive(Clone, Debug)]
pub struct TableSchema<C> {
    pub(crate) name: Ident,
    pub(crate) columns: C,
}

impl<C> TableSchema<C> {
    pub fn new(name: impl Into<Ident>, columns: C) -> Self {
        Self {
            name: name.into(),
            columns,
        }
    }
}
