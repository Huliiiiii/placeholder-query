use crate::{expr::Ident, projection::Projection};

pub trait Table {
    type Row;
    type Columns: Clone + Projection<Output = Self::Row>;

    const NAME: &'static str;

    fn bind_alias(alias: Ident) -> Self::Columns;
}
