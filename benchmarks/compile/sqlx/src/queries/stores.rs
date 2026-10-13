use crate::{Staff, StaffWithManager, StoreContact};

use super::Select;

pub fn staff_for_store(store_id: i32) -> Select<Staff> {
    sqlx::query_as(
        "SELECT staff_id, first_name, last_name, address_id, email, store_id, active, username,
                password, last_update, picture
         FROM staff
         WHERE store_id = $1
         ORDER BY staff_id
         LIMIT 50",
    )
    .bind(store_id)
}

pub fn store_contacts() -> Select<StoreContact> {
    sqlx::query_as(
        "SELECT store.store_id, staff.first_name AS manager_first_name,
                staff.last_name AS manager_last_name, address.phone, address.address,
                city.city, country.country
         FROM store
         INNER JOIN staff ON staff.staff_id = store.manager_staff_id
         INNER JOIN address ON address.address_id = store.address_id
         INNER JOIN city ON city.city_id = address.city_id
         INNER JOIN country ON country.country_id = city.country_id
         ORDER BY store.store_id",
    )
}

pub fn staff_with_managers(store_id: i32) -> Select<StaffWithManager> {
    sqlx::query_as(
        "SELECT employee.staff_id, employee.first_name, employee.last_name,
                store.manager_staff_id AS manager_id,
                manager.first_name AS manager_first_name,
                manager.last_name AS manager_last_name
         FROM staff AS employee
         INNER JOIN store ON store.store_id = employee.store_id
         INNER JOIN staff AS manager ON manager.staff_id = store.manager_staff_id
         WHERE employee.store_id = $1
         ORDER BY employee.staff_id
         LIMIT 50",
    )
    .bind(store_id)
}
