use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{Query, QueryFragment},
};

use crate::schema::{
    actor, address, category, city, country, customer, film, film_actor, film_category, inventory,
    language, payment, rental, staff, store,
};

pub fn all_actors() -> impl Query<SqlType = actor::SqlType> + QueryFragment<Pg> {
    actor::table.select(actor::all_columns)
}

pub fn all_films() -> impl Query<SqlType = film::SqlType> + QueryFragment<Pg> {
    film::table.select(film::all_columns)
}

pub fn all_languages() -> impl Query<SqlType = language::SqlType> + QueryFragment<Pg> {
    language::table.select(language::all_columns)
}

pub fn all_film_actors() -> impl Query<SqlType = film_actor::SqlType> + QueryFragment<Pg> {
    film_actor::table.select(film_actor::all_columns)
}

pub fn all_categories() -> impl Query<SqlType = category::SqlType> + QueryFragment<Pg> {
    category::table.select(category::all_columns)
}

pub fn all_film_categories() -> impl Query<SqlType = film_category::SqlType> + QueryFragment<Pg> {
    film_category::table.select(film_category::all_columns)
}

pub fn all_addresses() -> impl Query<SqlType = address::SqlType> + QueryFragment<Pg> {
    address::table.select(address::all_columns)
}

pub fn all_cities() -> impl Query<SqlType = city::SqlType> + QueryFragment<Pg> {
    city::table.select(city::all_columns)
}

pub fn all_countries() -> impl Query<SqlType = country::SqlType> + QueryFragment<Pg> {
    country::table.select(country::all_columns)
}

pub fn all_customers() -> impl Query<SqlType = customer::SqlType> + QueryFragment<Pg> {
    customer::table.select(customer::all_columns)
}

pub fn all_inventory() -> impl Query<SqlType = inventory::SqlType> + QueryFragment<Pg> {
    inventory::table.select(inventory::all_columns)
}

pub fn all_rentals() -> impl Query<SqlType = rental::SqlType> + QueryFragment<Pg> {
    rental::table.select(rental::all_columns)
}

pub fn all_payments() -> impl Query<SqlType = payment::SqlType> + QueryFragment<Pg> {
    payment::table.select(payment::all_columns)
}

pub fn all_staff() -> impl Query<SqlType = staff::SqlType> + QueryFragment<Pg> {
    staff::table.select(staff::all_columns)
}

pub fn all_stores() -> impl Query<SqlType = store::SqlType> + QueryFragment<Pg> {
    store::table.select(store::all_columns)
}
