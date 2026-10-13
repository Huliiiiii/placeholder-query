use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{Query, QueryFragment},
};

use diesel::sql_types::{Integer, Nullable, Text};

use crate::schema::{address, city, country, customer};

pub fn customers_by_store(
    store_id: i32,
    active: bool,
) -> impl Query<SqlType = customer::SqlType> + QueryFragment<Pg> {
    customer::table
        .filter(
            customer::store_id
                .eq(store_id)
                .and(customer::activebool.eq(active)),
        )
        .select(customer::all_columns)
        .order((
            customer::last_name,
            customer::first_name,
            customer::customer_id,
        ))
        .limit(50)
}

pub fn customer_contacts(
    store_id: i32,
) -> impl Query<
    SqlType = (
        Integer,
        Text,
        Text,
        Nullable<Text>,
        Text,
        Nullable<Text>,
        Text,
        Nullable<Text>,
        Text,
        Text,
        Text,
    ),
> + QueryFragment<Pg> {
    customer::table
        .inner_join(address::table.on(customer::address_id.eq(address::address_id)))
        .inner_join(city::table.on(address::city_id.eq(city::city_id)))
        .inner_join(country::table.on(city::country_id.eq(country::country_id)))
        .filter(customer::store_id.eq(store_id))
        .select((
            customer::customer_id,
            customer::first_name,
            customer::last_name,
            customer::email,
            address::address_,
            address::address2,
            address::district,
            address::postal_code,
            city::city_,
            country::country_,
            address::phone,
        ))
        .order(customer::customer_id)
        .limit(50)
}
