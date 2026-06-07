use crate::{expr::Ident, projection::Projection};

pub trait Table {
    type Row;
    type Ref: Clone + Projection<Output = Self::Row>;

    const NAME: &'static str;

    fn bind(alias: Ident) -> Self::Ref;
}
