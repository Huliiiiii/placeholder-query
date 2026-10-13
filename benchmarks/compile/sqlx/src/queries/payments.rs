use crate::{Payment, PaymentReceipt};

use super::Select;

pub fn customer_payments(customer_id: i32) -> Select<Payment> {
    sqlx::query_as(
        "SELECT payment_id, customer_id, staff_id, rental_id, amount, payment_date
         FROM payment
         WHERE customer_id = $1
         ORDER BY payment_date DESC, payment_id DESC
         LIMIT 50",
    )
    .bind(customer_id)
}

pub fn rental_receipts(customer_id: i32) -> Select<PaymentReceipt> {
    sqlx::query_as(
        "SELECT payment.payment_id, payment.rental_id, payment.amount, payment.payment_date,
                film.title, customer.first_name AS customer_first_name,
                customer.last_name AS customer_last_name, staff.first_name AS staff_first_name,
                staff.last_name AS staff_last_name
         FROM payment
         INNER JOIN rental ON rental.rental_id = payment.rental_id
         INNER JOIN inventory ON inventory.inventory_id = rental.inventory_id
         INNER JOIN film ON film.film_id = inventory.film_id
         INNER JOIN customer ON customer.customer_id = payment.customer_id
         INNER JOIN staff ON staff.staff_id = payment.staff_id
         WHERE payment.customer_id = $1
         ORDER BY payment.payment_date DESC, payment.payment_id DESC
         LIMIT 50",
    )
    .bind(customer_id)
}
