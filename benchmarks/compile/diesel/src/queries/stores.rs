use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{Query, QueryFragment},
};

use diesel::sql_types::{Integer, Text};

use crate::schema::{address, city, country, staff, store};

pub fn staff_for_store(store_id: i32) -> impl Query<SqlType = staff::SqlType> + QueryFragment<Pg> {
    staff::table
        .filter(staff::store_id.eq(store_id))
        .select(staff::all_columns)
        .order(staff::staff_id)
        .limit(50)
}

pub fn store_contacts()
-> impl Query<SqlType = (Integer, Text, Text, Text, Text, Text, Text)> + QueryFragment<Pg> {
    store::table
        .inner_join(staff::table.on(store::manager_staff_id.eq(staff::staff_id)))
        .inner_join(address::table.on(store::address_id.eq(address::address_id)))
        .inner_join(city::table.on(address::city_id.eq(city::city_id)))
        .inner_join(country::table.on(city::country_id.eq(country::country_id)))
        .select((
            store::store_id,
            staff::first_name,
            staff::last_name,
            address::phone,
            address::address_,
            city::city_,
            country::country_,
        ))
        .order(store::store_id)
}

pub fn staff_with_managers(
    store_id: i32,
) -> impl Query<SqlType = (Integer, Text, Text, Integer, Text, Text)> + QueryFragment<Pg> {
    let (employee, manager) = diesel::alias!(staff as employee, staff as manager);

    employee
        .inner_join(store::table.on(employee.field(staff::store_id).eq(store::store_id)))
        .inner_join(manager.on(store::manager_staff_id.eq(manager.field(staff::staff_id))))
        .filter(employee.field(staff::store_id).eq(store_id))
        .select((
            employee.field(staff::staff_id),
            employee.field(staff::first_name),
            employee.field(staff::last_name),
            manager.field(staff::staff_id),
            manager.field(staff::first_name),
            manager.field(staff::last_name),
        ))
        .order(employee.field(staff::staff_id))
        .limit(50)
}
