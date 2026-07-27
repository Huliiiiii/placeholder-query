use crate::{backend::QueryBackend, ident::TableAlias, projection::Projection};

pub trait Table<B: QueryBackend> {
    type Row;
    type Columns: Clone + Projection<B, Output = Self::Row>;

    const NAME: &'static str;

    fn bind_alias(alias: TableAlias) -> Self::Columns;
}
