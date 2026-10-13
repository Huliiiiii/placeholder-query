use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use placeholder_query_macro::table;

#[table(name = "actor", derive(Clone, Debug))]
pub struct Actor {
    pub actor_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub last_update: DateTime<Utc>,
}

#[table(name = "film", derive(Clone, Debug))]
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

#[table(name = "language", derive(Clone, Debug))]
pub struct Language {
    pub language_id: i32,
    pub name: String,
    pub last_update: DateTime<Utc>,
}

#[table(name = "film_actor", derive(Clone, Debug))]
pub struct FilmActor {
    pub actor_id: i32,
    pub film_id: i32,
    pub last_update: DateTime<Utc>,
}

#[table(name = "category", derive(Clone, Debug))]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub last_update: DateTime<Utc>,
}

#[table(name = "film_category", derive(Clone, Debug))]
pub struct FilmCategory {
    pub film_id: i32,
    pub category_id: i32,
    pub last_update: DateTime<Utc>,
}

#[table(name = "address", derive(Clone, Debug))]
pub struct Address {
    pub address_id: i32,
    pub address: String,
    pub address2: Option<String>,
    pub district: String,
    pub city_id: i32,
    pub postal_code: Option<String>,
    pub phone: String,
    pub last_update: DateTime<Utc>,
}

#[table(name = "city", derive(Clone, Debug))]
pub struct City {
    pub city_id: i32,
    pub city: String,
    pub country_id: i32,
    pub last_update: DateTime<Utc>,
}

#[table(name = "country", derive(Clone, Debug))]
pub struct Country {
    pub country_id: i32,
    pub country: String,
    pub last_update: DateTime<Utc>,
}

#[table(name = "customer", derive(Clone, Debug))]
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

#[table(name = "inventory", derive(Clone, Debug))]
pub struct Inventory {
    pub inventory_id: i32,
    pub film_id: i32,
    pub store_id: i32,
    pub last_update: DateTime<Utc>,
}

#[table(name = "rental", derive(Clone, Debug))]
pub struct Rental {
    pub rental_id: i32,
    pub rental_date: DateTime<Utc>,
    pub inventory_id: i32,
    pub customer_id: i32,
    pub return_date: Option<DateTime<Utc>>,
    pub staff_id: i32,
    pub last_update: DateTime<Utc>,
}

#[table(name = "payment", derive(Clone, Debug))]
pub struct Payment {
    pub payment_id: i32,
    pub customer_id: i32,
    pub staff_id: i32,
    pub rental_id: i32,
    pub amount: BigDecimal,
    pub payment_date: DateTime<Utc>,
}

#[table(name = "staff", derive(Clone, Debug))]
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

#[table(name = "store", derive(Clone, Debug))]
pub struct Store {
    pub store_id: i32,
    pub manager_staff_id: i32,
    pub address_id: i32,
    pub last_update: DateTime<Utc>,
}
