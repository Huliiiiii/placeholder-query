use crate::{CustomerLatestRental, RecentRentalCustomer, RentalHistory, StockItem};

use super::Select;

pub fn inventory_for_store(store_id: i32) -> Select<StockItem> {
    sqlx::query_as(
        "SELECT inventory.inventory_id, inventory.film_id, film.title, inventory.store_id
         FROM inventory
         INNER JOIN film ON film.film_id = inventory.film_id
         WHERE inventory.store_id = $1
         ORDER BY film.title, inventory.inventory_id
         LIMIT 50",
    )
    .bind(store_id)
}

pub fn customer_rental_history(customer_id: i32) -> Select<RentalHistory> {
    sqlx::query_as(
        "SELECT rental.rental_id, inventory.film_id, film.title, rental.rental_date, rental.return_date
         FROM rental
         INNER JOIN inventory ON inventory.inventory_id = rental.inventory_id
         INNER JOIN film ON film.film_id = inventory.film_id
         WHERE rental.customer_id = $1
         ORDER BY rental.rental_date DESC, rental.rental_id DESC
         LIMIT 50",
    )
    .bind(customer_id)
}

pub fn recent_rental_customers() -> Select<RecentRentalCustomer> {
    sqlx::query_as(
        "SELECT recent.rental_id, recent.customer_id, customer.first_name,
                customer.last_name, recent.rental_date
         FROM (
             SELECT rental_id, customer_id, rental_date
             FROM rental
             ORDER BY rental_date DESC, rental_id DESC
             LIMIT 50
         ) AS recent
         INNER JOIN customer ON customer.customer_id = recent.customer_id
         ORDER BY recent.rental_date DESC, recent.rental_id DESC",
    )
}

pub fn customers_with_latest_rental(store_id: i32) -> Select<CustomerLatestRental> {
    sqlx::query_as(
        "SELECT customer.customer_id, customer.first_name, customer.last_name, latest.rental_id
         FROM customer
         LEFT JOIN LATERAL (
             SELECT rental.rental_id
             FROM rental
             WHERE rental.customer_id = customer.customer_id
             ORDER BY rental.rental_date DESC, rental.rental_id DESC
             LIMIT 1
         ) AS latest ON TRUE
         WHERE customer.store_id = $1
         ORDER BY customer.customer_id
         LIMIT 50",
    )
    .bind(store_id)
}
