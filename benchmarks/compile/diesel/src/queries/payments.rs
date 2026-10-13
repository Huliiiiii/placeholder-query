use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{Query, QueryFragment},
};

use diesel::sql_types::{Integer, Numeric, Text, Timestamptz};

use crate::schema::{customer, film, inventory, payment, rental, staff};

pub fn customer_payments(
    customer_id: i32,
) -> impl Query<SqlType = payment::SqlType> + QueryFragment<Pg> {
    payment::table
        .filter(payment::customer_id.eq(customer_id))
        .select(payment::all_columns)
        .order((payment::payment_date.desc(), payment::payment_id.desc()))
        .limit(50)
}

pub fn rental_receipts(
    customer_id: i32,
) -> impl Query<
    SqlType = (
        Integer,
        Integer,
        Numeric,
        Timestamptz,
        Text,
        Text,
        Text,
        Text,
        Text,
    ),
> + QueryFragment<Pg> {
    payment::table
        .inner_join(rental::table.on(payment::rental_id.eq(rental::rental_id)))
        .inner_join(inventory::table.on(rental::inventory_id.eq(inventory::inventory_id)))
        .inner_join(film::table.on(inventory::film_id.eq(film::film_id)))
        .inner_join(customer::table.on(payment::customer_id.eq(customer::customer_id)))
        .inner_join(staff::table.on(payment::staff_id.eq(staff::staff_id)))
        .filter(payment::customer_id.eq(customer_id))
        .select((
            payment::payment_id,
            payment::rental_id,
            payment::amount,
            payment::payment_date,
            film::title,
            customer::first_name,
            customer::last_name,
            staff::first_name,
            staff::last_name,
        ))
        .order((payment::payment_date.desc(), payment::payment_id.desc()))
        .limit(50)
}
