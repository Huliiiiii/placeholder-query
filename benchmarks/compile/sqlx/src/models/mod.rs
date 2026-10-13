#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Actor {
    pub actor_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Film {
    pub film_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub release_year: Option<i32>,
    pub language_id: i32,
    pub original_language_id: Option<i32>,
    pub rental_duration: i16,
    pub rental_rate: bigdecimal::BigDecimal,
    pub length: Option<i16>,
    pub replacement_cost: bigdecimal::BigDecimal,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub special_features: Option<Vec<Option<String>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Language {
    pub language_id: i32,
    pub name: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct FilmActor {
    pub actor_id: i32,
    pub film_id: i32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Category {
    pub category_id: i32,
    pub name: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct FilmCategory {
    pub film_id: i32,
    pub category_id: i32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Address {
    pub address_id: i32,
    pub address: String,
    pub address2: Option<String>,
    pub district: String,
    pub city_id: i32,
    pub postal_code: Option<String>,
    pub phone: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct City {
    pub city_id: i32,
    pub city: String,
    pub country_id: i32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Country {
    pub country_id: i32,
    pub country: String,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Customer {
    pub customer_id: i32,
    pub store_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub address_id: i32,
    pub activebool: bool,
    pub create_date: chrono::NaiveDate,
    pub last_update: Option<chrono::DateTime<chrono::Utc>>,
    pub active: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Inventory {
    pub inventory_id: i32,
    pub film_id: i32,
    pub store_id: i32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Rental {
    pub rental_id: i32,
    pub rental_date: chrono::DateTime<chrono::Utc>,
    pub inventory_id: i32,
    pub customer_id: i32,
    pub return_date: Option<chrono::DateTime<chrono::Utc>>,
    pub staff_id: i32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Payment {
    pub payment_id: i32,
    pub customer_id: i32,
    pub staff_id: i32,
    pub rental_id: i32,
    pub amount: bigdecimal::BigDecimal,
    pub payment_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
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
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub picture: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct Store {
    pub store_id: i32,
    pub manager_staff_id: i32,
    pub address_id: i32,
    pub last_update: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct FilmLanguage {
    pub film_id: i32,
    pub title: String,
    pub language_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct FilmListing {
    pub film_id: i32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct ActorListing {
    pub actor_id: i32,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct StockItem {
    pub inventory_id: i32,
    pub film_id: i32,
    pub title: String,
    pub store_id: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct RentalHistory {
    pub rental_id: i32,
    pub film_id: i32,
    pub title: String,
    pub rental_date: chrono::DateTime<chrono::Utc>,
    pub return_date: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct CustomerContact {
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub address: String,
    pub address2: Option<String>,
    pub district: String,
    pub postal_code: Option<String>,
    pub city: String,
    pub country: String,
    pub phone: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct StoreContact {
    pub store_id: i32,
    pub manager_first_name: String,
    pub manager_last_name: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub country: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct PaymentReceipt {
    pub payment_id: i32,
    pub rental_id: i32,
    pub amount: bigdecimal::BigDecimal,
    pub payment_date: chrono::DateTime<chrono::Utc>,
    pub title: String,
    pub customer_first_name: String,
    pub customer_last_name: String,
    pub staff_first_name: String,
    pub staff_last_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct CatalogEntry {
    pub film_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub rental_rate: bigdecimal::BigDecimal,
    pub language_name: String,
    pub category_id: i32,
    pub category_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct FilmInventory {
    pub film_id: i32,
    pub title: String,
    pub inventory_id: Option<i32>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct StaffWithManager {
    pub staff_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub manager_id: i32,
    pub manager_first_name: String,
    pub manager_last_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct RecentRentalCustomer {
    pub rental_id: i32,
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub rental_date: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq, sqlx::FromRow)]
pub struct CustomerLatestRental {
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub rental_id: Option<i32>,
}
