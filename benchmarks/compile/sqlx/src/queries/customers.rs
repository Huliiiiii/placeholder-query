use crate::{Customer, CustomerContact};

use super::Select;

pub fn customers_by_store(store_id: i32, active: bool) -> Select<Customer> {
    sqlx::query_as(
        "SELECT customer_id, store_id, first_name, last_name, email, address_id, activebool,
                create_date, last_update, active
         FROM customer
         WHERE store_id = $1 AND activebool = $2
         ORDER BY last_name, first_name, customer_id
         LIMIT 50",
    )
    .bind(store_id)
    .bind(active)
}

pub fn customer_contacts(store_id: i32) -> Select<CustomerContact> {
    sqlx::query_as(
        "SELECT customer.customer_id, customer.first_name, customer.last_name, customer.email,
                address.address, address.address2, address.district, address.postal_code,
                city.city, country.country, address.phone
         FROM customer
         INNER JOIN address ON address.address_id = customer.address_id
         INNER JOIN city ON city.city_id = address.city_id
         INNER JOIN country ON country.country_id = city.country_id
         WHERE customer.store_id = $1
         ORDER BY customer.customer_id
         LIMIT 50",
    )
    .bind(store_id)
}
