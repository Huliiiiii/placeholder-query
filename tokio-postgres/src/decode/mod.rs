use placeholder_query_postgres::query::projection::Scalar;
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

        pub(super) fn advance(&mut self, by: usize) {
            self.index += by;
        }
    }
}

pub trait DecodeRow: Sealed + Sized {
    const ARITY: usize;

    /// Decodes `Self` from the decoder.
    ///
    /// On error, columns already read remain consumed.
    fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error>;
}

impl<T> Sealed for Scalar<T> {}

impl<T: FromSqlOwned> DecodeRow for Scalar<T> {
    const ARITY: usize = 1;

    fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error> {
        Ok(Scalar(decoder.read_next()?))
    }
}

impl<T> Sealed for Option<T> where T: DecodeRow {}

impl<T: DecodeRow> DecodeRow for Option<T> {
    const ARITY: usize = 1 + T::ARITY;

    fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error> {
        if decoder.read_next::<bool>()? {
            Ok(Some(T::decode(decoder)?))
        } else {
            decoder.advance(T::ARITY);
            Ok(None)
        }
    }
}

impl Sealed for () {}

impl DecodeRow for () {
    const ARITY: usize = 0;

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
        const ARITY: usize = #(#Member::ARITY)+*;

        fn decode(decoder: &mut RowDecoder<'_>) -> Result<Self, Error> {
            Ok((#(#Member::decode(decoder)?,)*))
        }
    }
}
