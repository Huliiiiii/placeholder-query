use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{Query, QueryFragment},
};

use diesel::sql_types::{Integer, Nullable, Text, Timestamptz};

use crate::schema::{customer, film, inventory, rental};

pub fn inventory_for_store(
    store_id: i32,
) -> impl Query<SqlType = (Integer, Integer, Text, Integer)> + QueryFragment<Pg> {
    inventory::table
        .inner_join(film::table.on(inventory::film_id.eq(film::film_id)))
        .filter(inventory::store_id.eq(store_id))
        .select((
            inventory::inventory_id,
            film::film_id,
            film::title,
            inventory::store_id,
        ))
        .order((film::title, inventory::inventory_id))
        .limit(50)
}

pub fn customer_rental_history(
    customer_id: i32,
) -> impl Query<SqlType = (Integer, Integer, Text, Timestamptz, Nullable<Timestamptz>)> + QueryFragment<Pg>
{
    rental::table
        .inner_join(inventory::table.on(rental::inventory_id.eq(inventory::inventory_id)))
        .inner_join(film::table.on(inventory::film_id.eq(film::film_id)))
        .filter(rental::customer_id.eq(customer_id))
        .select((
            rental::rental_id,
            film::film_id,
            film::title,
            rental::rental_date,
            rental::return_date,
        ))
        .order((rental::rental_date.desc(), rental::rental_id.desc()))
        .limit(50)
}

pub fn recent_rental_customers()
-> impl Query<SqlType = (Integer, Integer, Text, Text, Timestamptz)> + QueryFragment<Pg> {
    let recent = diesel::alias!(rental as recent);

    rental::table
        .inner_join(customer::table.on(rental::customer_id.eq(customer::customer_id)))
        .filter(
            rental::rental_id.eq_any(
                recent
                    .select(recent.field(rental::rental_id))
                    .order((
                        recent.field(rental::rental_date).desc(),
                        recent.field(rental::rental_id).desc(),
                    ))
                    .limit(50),
            ),
        )
        .select((
            rental::rental_id,
            customer::customer_id,
            customer::first_name,
            customer::last_name,
            rental::rental_date,
        ))
        .order((rental::rental_date.desc(), rental::rental_id.desc()))
}

pub fn customers_with_latest_rental(
    store_id: i32,
) -> impl Query<SqlType = (Integer, Text, Text, Nullable<Integer>)> + QueryFragment<Pg> {
    customer::table
        .filter(customer::store_id.eq(store_id))
        .select((
            customer::customer_id,
            customer::first_name,
            customer::last_name,
            rental::table
                .filter(rental::customer_id.eq(customer::customer_id))
                .order((rental::rental_date.desc(), rental::rental_id.desc()))
                .select(rental::rental_id)
                .single_value(),
        ))
        .order(customer::customer_id)
        .limit(50)
}
