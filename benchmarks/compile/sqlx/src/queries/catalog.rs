use crate::{ActorListing, CatalogEntry, Film, FilmInventory, FilmLanguage, FilmListing};

use super::Select;

pub fn films_by_language(language_id: i32) -> Select<Film> {
    sqlx::query_as(
        "SELECT film_id, title, description, release_year, language_id, original_language_id,
                rental_duration, rental_rate, length, replacement_cost, last_update, special_features
         FROM film
         WHERE language_id = $1
         ORDER BY title, film_id
         LIMIT 50",
    )
    .bind(language_id)
}

pub fn films_with_language() -> Select<FilmLanguage> {
    sqlx::query_as(
        "SELECT film.film_id, film.title, language.name AS language_name
         FROM film
         INNER JOIN language ON language.language_id = film.language_id
         ORDER BY film.title, film.film_id
         LIMIT 50",
    )
}

pub fn films_by_actor(actor_id: i32) -> Select<FilmListing> {
    sqlx::query_as(
        "SELECT film.film_id, film.title
         FROM film
         INNER JOIN film_actor ON film_actor.film_id = film.film_id
         WHERE film_actor.actor_id = $1
         ORDER BY film.title, film.film_id
         LIMIT 50",
    )
    .bind(actor_id)
}

pub fn films_by_category(category_id: i32) -> Select<FilmListing> {
    sqlx::query_as(
        "SELECT film.film_id, film.title
         FROM film
         INNER JOIN film_category ON film_category.film_id = film.film_id
         WHERE film_category.category_id = $1
         ORDER BY film.title, film.film_id
         LIMIT 50",
    )
    .bind(category_id)
}

pub fn cast_for_film(film_id: i32) -> Select<ActorListing> {
    sqlx::query_as(
        "SELECT actor.actor_id, actor.first_name, actor.last_name
         FROM film_actor
         INNER JOIN actor ON actor.actor_id = film_actor.actor_id
         WHERE film_actor.film_id = $1
         ORDER BY actor.last_name, actor.first_name, actor.actor_id",
    )
    .bind(film_id)
}

pub fn film_catalog() -> Select<CatalogEntry> {
    sqlx::query_as(
        "SELECT film.film_id, film.title, film.description, film.rental_rate,
                language.name AS language_name, category.category_id, category.name AS category_name
         FROM film
         INNER JOIN language ON language.language_id = film.language_id
         INNER JOIN film_category ON film_category.film_id = film.film_id
         INNER JOIN category ON category.category_id = film_category.category_id
         ORDER BY film.title, film.film_id, category.category_id
         LIMIT 50",
    )
}

pub fn search_films(title_pattern: String) -> Select<FilmListing> {
    sqlx::query_as(
        "SELECT film_id, title
         FROM film
         WHERE title LIKE $1
         ORDER BY title, film_id
         LIMIT 50",
    )
    .bind(title_pattern)
}

pub fn films_by_ids(film_ids: Vec<i32>) -> Select<FilmListing> {
    sqlx::query_as(
        "SELECT film_id, title
         FROM film
         WHERE film_id = ANY($1::integer[])
         ORDER BY film_id ASC",
    )
    .bind(film_ids)
}

pub fn filter_films(
    language_id: i32,
    min_rental_duration: i16,
    max_rental_duration: i16,
    title_pattern: String,
    alternative_title_pattern: String,
) -> Select<FilmListing> {
    sqlx::query_as(
        "SELECT film_id, title
         FROM film
         WHERE language_id = $1
           AND rental_duration >= $2
           AND rental_duration <= $3
           AND (title LIKE $4 OR title LIKE $5)
         ORDER BY title, film_id
         LIMIT 50",
    )
    .bind(language_id)
    .bind(min_rental_duration)
    .bind(max_rental_duration)
    .bind(title_pattern)
    .bind(alternative_title_pattern)
}

pub fn films_after(title: String, film_id: i32) -> Select<FilmListing> {
    sqlx::query_as(
        "SELECT film_id, title
         FROM film
         WHERE title > $1 OR (title = $1 AND film_id > $2)
         ORDER BY title, film_id
         LIMIT 50",
    )
    .bind(title)
    .bind(film_id)
}

pub fn films_with_store_inventory(store_id: i32) -> Select<FilmInventory> {
    sqlx::query_as(
        "SELECT film.film_id, film.title, inventory.inventory_id
         FROM film
         LEFT JOIN inventory
           ON inventory.film_id = film.film_id AND inventory.store_id = $1
         ORDER BY film.film_id, inventory.inventory_id ASC NULLS LAST
         LIMIT 50",
    )
    .bind(store_id)
}
