use tokio_postgres::types::ToSql;

pub trait BindParams {
    fn params(&self) -> impl Iterator<Item = &(dyn ToSql + Sync)>;
}

impl BindParams for () {
    fn params(&self) -> impl Iterator<Item = &(dyn ToSql + Sync)> {
        std::iter::empty()
    }
}

fortuples::fortuples! {
    #[tuples::min_size(1)]
    #[tuples::max_size(21)]
    impl BindParams for #Tuple
    where #(#Member: ToSql + Sync),* {
        fn params(&self) -> impl Iterator<Item = &(dyn ToSql + Sync)> {
            [#(&#self as &(dyn ToSql + Sync),)*].into_iter()
        }
    }
}
