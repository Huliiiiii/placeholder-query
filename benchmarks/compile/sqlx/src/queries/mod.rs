mod catalog;
mod customers;
mod payments;
mod rentals;
mod stores;
mod tables;

pub use catalog::*;
pub use customers::*;
pub use payments::*;
pub use rentals::*;
pub use stores::*;
pub use tables::*;

use sqlx::{Postgres, postgres::PgArguments, query::QueryAs};

pub type Select<T> = QueryAs<'static, Postgres, T, PgArguments>;
