use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;

use crate::schema::*;

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = actor)]
pub struct Actor {
    pub actor_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = film)]
pub struct Film {
    pub film_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub language_id: i32,
    pub original_language_id: Option<i32>,
    pub rental_duration: i16,
    pub rental_rate: BigDecimal,
    pub length: Option<i16>,
    pub replacement_cost: BigDecimal,
    pub last_update: DateTime<Utc>,
    pub special_features: Option<Vec<Option<String>>>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = language)]
pub struct Language {
    pub language_id: i32,
    pub name: String,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = film_actor)]
pub struct FilmActor {
    pub actor_id: i32,
    pub film_id: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = category)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = film_category)]
pub struct FilmCategory {
    pub film_id: i32,
    pub category_id: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = address)]
pub struct Address {
    pub address_id: i32,
    #[diesel(column_name = address_)]
    pub address: String,
    pub address2: Option<String>,
    pub district: String,
    pub city_id: i32,
    pub postal_code: Option<String>,
    pub phone: String,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = city)]
pub struct City {
    pub city_id: i32,
    #[diesel(column_name = city_)]
    pub city: String,
    pub country_id: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = country)]
pub struct Country {
    pub country_id: i32,
    #[diesel(column_name = country_)]
    pub country: String,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = customer)]
pub struct Customer {
    pub customer_id: i32,
    pub store_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub address_id: i32,
    pub activebool: bool,
    pub create_date: NaiveDate,
    pub last_update: Option<DateTime<Utc>>,
    pub active: Option<i32>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = inventory)]
pub struct Inventory {
    pub inventory_id: i32,
    pub film_id: i32,
    pub store_id: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = rental)]
pub struct Rental {
    pub rental_id: i32,
    pub rental_date: DateTime<Utc>,
    pub inventory_id: i32,
    pub customer_id: i32,
    pub return_date: Option<DateTime<Utc>>,
    pub staff_id: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = payment)]
pub struct Payment {
    pub payment_id: i32,
    pub customer_id: i32,
    pub staff_id: i32,
    pub rental_id: i32,
    pub amount: BigDecimal,
    pub payment_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = staff)]
pub struct Staff {
    pub staff_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub address_id: i32,
    pub email: Option<String>,
    pub store_id: i32,
    pub active: bool,
    pub username: String,
    pub password: Option<String>,
    pub last_update: DateTime<Utc>,
    pub picture: Option<Vec<u8>>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
#[diesel(table_name = store)]
pub struct Store {
    pub store_id: i32,
    pub manager_staff_id: i32,
    pub address_id: i32,
    pub last_update: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct FilmLanguage {
    #[diesel(select_expression = film::film_id)]
    pub film_id: i32,
    #[diesel(select_expression = film::title)]
    pub title: String,
    #[diesel(select_expression = language::name)]
    pub language_name: String,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct FilmListing {
    #[diesel(select_expression = film::film_id)]
    pub film_id: i32,
    #[diesel(select_expression = film::title)]
    pub title: String,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct ActorListing {
    #[diesel(select_expression = actor::actor_id)]
    pub actor_id: i32,
    #[diesel(select_expression = actor::first_name)]
    pub first_name: String,
    #[diesel(select_expression = actor::last_name)]
    pub last_name: String,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct StockItem {
    #[diesel(select_expression = inventory::inventory_id)]
    pub inventory_id: i32,
    #[diesel(select_expression = film::film_id)]
    pub film_id: i32,
    #[diesel(select_expression = film::title)]
    pub title: String,
    #[diesel(select_expression = inventory::store_id)]
    pub store_id: i32,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct RentalHistory {
    #[diesel(select_expression = rental::rental_id)]
    pub rental_id: i32,
    #[diesel(select_expression = film::film_id)]
    pub film_id: i32,
    #[diesel(select_expression = film::title)]
    pub title: String,
    #[diesel(select_expression = rental::rental_date)]
    pub rental_date: DateTime<Utc>,
    #[diesel(select_expression = rental::return_date)]
    pub return_date: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct CustomerContact {
    #[diesel(select_expression = customer::customer_id)]
    pub customer_id: i32,
    #[diesel(select_expression = customer::first_name)]
    pub first_name: String,
    #[diesel(select_expression = customer::last_name)]
    pub last_name: String,
    #[diesel(select_expression = customer::email)]
    pub email: Option<String>,
    #[diesel(select_expression = address::address_)]
    pub address: String,
    #[diesel(select_expression = address::address2)]
    pub address2: Option<String>,
    #[diesel(select_expression = address::district)]
    pub district: String,
    #[diesel(select_expression = address::postal_code)]
    pub postal_code: Option<String>,
    #[diesel(select_expression = city::city_)]
    pub city: String,
    #[diesel(select_expression = country::country_)]
    pub country: String,
    #[diesel(select_expression = address::phone)]
    pub phone: String,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct StoreContact {
    #[diesel(select_expression = store::store_id)]
    pub store_id: i32,
    #[diesel(select_expression = staff::first_name)]
    pub manager_first_name: String,
    #[diesel(select_expression = staff::last_name)]
    pub manager_last_name: String,
    #[diesel(select_expression = address::phone)]
    pub phone: String,
    #[diesel(select_expression = address::address_)]
    pub address: String,
    #[diesel(select_expression = city::city_)]
    pub city: String,
    #[diesel(select_expression = country::country_)]
    pub country: String,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct PaymentReceipt {
    #[diesel(select_expression = payment::payment_id)]
    pub payment_id: i32,
    #[diesel(select_expression = payment::rental_id)]
    pub rental_id: i32,
    #[diesel(select_expression = payment::amount)]
    pub amount: BigDecimal,
    #[diesel(select_expression = payment::payment_date)]
    pub payment_date: DateTime<Utc>,
    #[diesel(select_expression = film::title)]
    pub title: String,
    #[diesel(select_expression = customer::first_name)]
    pub customer_first_name: String,
    #[diesel(select_expression = customer::last_name)]
    pub customer_last_name: String,
    #[diesel(select_expression = staff::first_name)]
    pub staff_first_name: String,
    #[diesel(select_expression = staff::last_name)]
    pub staff_last_name: String,
}

#[derive(Clone, Debug, Queryable, Selectable)]
pub struct CatalogEntry {
    #[diesel(select_expression = film::film_id)]
    pub film_id: i32,
    #[diesel(select_expression = film::title)]
    pub title: String,
    #[diesel(select_expression = film::description)]
    pub description: Option<String>,
    #[diesel(select_expression = film::rental_rate)]
    pub rental_rate: BigDecimal,
    #[diesel(select_expression = language::name)]
    pub language_name: String,
    #[diesel(select_expression = category::category_id)]
    pub category_id: i32,
    #[diesel(select_expression = category::name)]
    pub category_name: String,
}

#[derive(Clone, Debug, Queryable)]
pub struct FilmInventory {
    pub film_id: i32,
    pub title: String,
    pub inventory_id: Option<i32>,
}

#[derive(Clone, Debug, Queryable)]
pub struct StaffWithManager {
    pub staff_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub manager_id: i32,
    pub manager_first_name: String,
    pub manager_last_name: String,
}

#[derive(Clone, Debug, Queryable)]
pub struct RecentRentalCustomer {
    pub rental_id: i32,
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub rental_date: DateTime<Utc>,
}

#[derive(Clone, Debug, Queryable)]
pub struct CustomerLatestRental {
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub rental_id: Option<i32>,
}
