use crate::{
    Actor, Address, Category, City, Country, Customer, Film, FilmActor, FilmCategory, Inventory,
    Language, Payment, Rental, Staff, Store,
};

use super::Select;

pub fn all_actors() -> Select<Actor> {
    sqlx::query_as("SELECT actor_id, first_name, last_name, last_update FROM actor")
}

pub fn all_films() -> Select<Film> {
    sqlx::query_as(
        "SELECT film_id, title, description, release_year, language_id, original_language_id,
                rental_duration, rental_rate, length, replacement_cost, last_update, special_features
         FROM film",
    )
}

pub fn all_languages() -> Select<Language> {
    sqlx::query_as("SELECT language_id, name, last_update FROM language")
}

pub fn all_film_actors() -> Select<FilmActor> {
    sqlx::query_as("SELECT actor_id, film_id, last_update FROM film_actor")
}

pub fn all_categories() -> Select<Category> {
    sqlx::query_as("SELECT category_id, name, last_update FROM category")
}

pub fn all_film_categories() -> Select<FilmCategory> {
    sqlx::query_as("SELECT film_id, category_id, last_update FROM film_category")
}

pub fn all_addresses() -> Select<Address> {
    sqlx::query_as(
        "SELECT address_id, address, address2, district, city_id, postal_code, phone, last_update
         FROM address",
    )
}

pub fn all_cities() -> Select<City> {
    sqlx::query_as("SELECT city_id, city, country_id, last_update FROM city")
}

pub fn all_countries() -> Select<Country> {
    sqlx::query_as("SELECT country_id, country, last_update FROM country")
}

pub fn all_customers() -> Select<Customer> {
    sqlx::query_as(
        "SELECT customer_id, store_id, first_name, last_name, email, address_id, activebool,
                create_date, last_update, active
         FROM customer",
    )
}

pub fn all_inventory() -> Select<Inventory> {
    sqlx::query_as("SELECT inventory_id, film_id, store_id, last_update FROM inventory")
}

pub fn all_rentals() -> Select<Rental> {
    sqlx::query_as(
        "SELECT rental_id, rental_date, inventory_id, customer_id, return_date, staff_id, last_update
         FROM rental",
    )
}

pub fn all_payments() -> Select<Payment> {
    sqlx::query_as(
        "SELECT payment_id, customer_id, staff_id, rental_id, amount, payment_date FROM payment",
    )
}

pub fn all_staff() -> Select<Staff> {
    sqlx::query_as(
        "SELECT staff_id, first_name, last_name, address_id, email, store_id, active, username,
                password, last_update, picture
         FROM staff",
    )
}

pub fn all_stores() -> Select<Store> {
    sqlx::query_as("SELECT store_id, manager_staff_id, address_id, last_update FROM store")
}
