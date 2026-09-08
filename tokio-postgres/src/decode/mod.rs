use placeholder_query_postgres::query::projection::ColumnValue;
use tokio_postgres::{Error, types::FromSqlOwned};

pub(super) use sealed::RowDecoder;
use sealed::Sealed;

mod sealed {
    use tokio_postgres::{Error, Row, types::FromSqlOwned};

    pub trait Sealed {}

    pub struct RowDecoder<'a> {
        row: &'a Row,
        index: usize,
    }

    impl<'a> RowDecoder<'a> {
        pub(crate) fn new(row: &'a Row) -> Self {
            Self { row, index: 0 }
        }

        /// Decodes the next column, advancing only on success.
        pub(super) fn read_next<T: FromSqlOwned>(&mut self) -> Result<T, Error> {
            let value = self.row.try_get(self.index)?;
            self.index += 1;
            Ok(value)
        }
    }
}

pub trait DecodeRow: Sealed + Sized {
    /// Decodes `Self` from the decoder.
    ///
    /// On error, columns already read remain consumed.
    fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error>;
}

impl<T> Sealed for ColumnValue<T> {}

impl<T: FromSqlOwned> DecodeRow for ColumnValue<T> {
    fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error> {
        Ok(ColumnValue(decoder.read_next()?))
    }
}

impl Sealed for () {}

impl DecodeRow for () {
    fn decode(_: &mut RowDecoder<'_>) -> Result<Self, Error> {
        Ok(())
    }
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl Sealed for #Tuple where #(#Member: Sealed),* {}
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl DecodeRow for #Tuple
    where #(#Member: DecodeRow),* {
        fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error> {
            Ok((#(#Member::decode(decoder)?,)*))
        }
    }
}
