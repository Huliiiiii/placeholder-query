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

use placeholder_query_macro::QueryInput;

#[derive(QueryInput)]
pub struct LanguageFilter {
    pub language_id: i32,
}

#[derive(QueryInput)]
pub struct ActorFilter {
    pub actor_id: i32,
}

#[derive(QueryInput)]
pub struct CategoryFilter {
    pub category_id: i32,
}

#[derive(QueryInput)]
pub struct FilmFilter {
    pub film_id: i32,
}

#[derive(QueryInput)]
pub struct StoreFilter {
    pub store_id: i32,
}

#[derive(QueryInput)]
pub struct CustomerFilter {
    pub customer_id: i32,
}

#[derive(QueryInput)]
pub struct StoreCustomerFilter {
    pub store_id: i32,
    pub active: bool,
}
