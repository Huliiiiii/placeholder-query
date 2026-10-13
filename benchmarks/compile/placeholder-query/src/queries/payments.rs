use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use placeholder_query_macro::row;
use placeholder_query_postgres::query::{mode, select as pg, select::template::SelectTemplate};

use super::CustomerFilter;
use crate::schema::{Customer, Film, Inventory, Payment, Rental, Staff};

#[row(derive(Clone, Debug))]
pub struct PaymentReceipt {
    pub payment_id: i32,
    pub rental_id: i32,
    pub amount: BigDecimal,
    pub payment_date: DateTime<Utc>,
    pub title: String,
    pub customer_first_name: String,
    pub customer_last_name: String,
    pub staff_first_name: String,
    pub staff_last_name: String,
}

pub fn customer_payments() -> SelectTemplate<Payment<mode::Expr>, CustomerFilter> {
    pg::query::<CustomerFilter, _>(|input| {
        pg::from(Payment::table())
            .filter(|payment| payment.customer_id().eq(input.customer_id))
            .order_by(|payment| payment.payment_date().desc())
            .order_by(|payment| payment.payment_id().desc())
            .limit(50)
    })
    .compile()
}

pub fn rental_receipts() -> SelectTemplate<PaymentReceipt<mode::Expr>, CustomerFilter> {
    pg::query::<CustomerFilter, _>(|input| {
        pg::from(Payment::table())
            .join(Rental::table())
            .on(|payment, rental| payment.rental_id().eq(rental.rental_id()))
            .join(Inventory::table())
            .on(|(_, rental), inventory| rental.inventory_id().eq(inventory.inventory_id()))
            .join(Film::table())
            .on(|(_, inventory), film| inventory.film_id().eq(film.film_id()))
            .map(|(((payment, _), _), film)| (payment, film))
            .join(Customer::table())
            .on(|(payment, _), customer| payment.customer_id().eq(customer.customer_id()))
            .join(Staff::table())
            .on(|((payment, _), _), staff| payment.staff_id().eq(staff.staff_id()))
            .filter(|(((payment, _), _), _)| payment.customer_id().eq(input.customer_id))
            .order_by(|(((payment, _), _), _)| payment.payment_date().desc())
            .order_by(|(((payment, _), _), _)| payment.payment_id().desc())
            .map(|(((payment, film), customer), staff)| PaymentReceipt {
                payment_id: payment.payment_id,
                rental_id: payment.rental_id,
                amount: payment.amount,
                payment_date: payment.payment_date,
                title: film.title,
                customer_first_name: customer.first_name,
                customer_last_name: customer.last_name,
                staff_first_name: staff.first_name,
                staff_last_name: staff.last_name,
            })
            .limit(50)
    })
    .compile()
}
