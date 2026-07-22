use crate::{backend::QueryBackend, ident::Ident, projection::Projection};

pub trait Table<B: QueryBackend> {
    type Row;
    type Columns: Clone + Projection<B, Output = Self::Row>;

    const NAME: &'static str;

    fn bind_alias(alias: Ident) -> Self::Columns;
}
