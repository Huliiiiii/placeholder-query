use placeholder_query_macro::row;
use placeholder_query_postgres::query::{mode, select as pg, select::template::SelectTemplate};

use super::StoreFilter;
use crate::schema::{Address, City, Country, Staff, Store};

#[row(derive(Clone, Debug))]
pub struct StoreContact {
    pub store_id: i32,
    pub manager_first_name: String,
    pub manager_last_name: String,
    pub phone: String,
    pub address: String,
    pub city: String,
    pub country: String,
}

pub fn staff_for_store() -> SelectTemplate<Staff<mode::Expr>, StoreFilter> {
    pg::query::<StoreFilter, _>(|input| {
        pg::from(Staff::table())
            .filter(|staff| staff.store_id().eq(input.store_id))
            .order_by(|staff| staff.staff_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn store_contacts() -> SelectTemplate<StoreContact<mode::Expr>> {
    pg::from(Store::table())
        .join(Staff::table())
        .on(|store, manager| store.manager_staff_id().eq(manager.staff_id()))
        .join(Address::table())
        .on(|(store, _), address| store.address_id().eq(address.address_id()))
        .join(City::table())
        .on(|(_, address), city| address.city_id().eq(city.city_id()))
        .join(Country::table())
        .on(|(_, city), country| city.country_id().eq(country.country_id()))
        .map(
            |((((store, manager), address), city), country)| StoreContact {
                store_id: store.store_id,
                manager_first_name: manager.first_name,
                manager_last_name: manager.last_name,
                phone: address.phone,
                address: address.address,
                city: city.city,
                country: country.country,
            },
        )
        .order_by(|store| store.store_id().asc())
        .compile()
}

#[row(derive(Clone, Debug))]
pub struct StaffWithManager {
    pub staff_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub manager_id: i32,
    pub manager_first_name: String,
    pub manager_last_name: String,
}

pub fn staff_with_managers() -> SelectTemplate<StaffWithManager<mode::Expr>, StoreFilter> {
    pg::query::<StoreFilter, _>(|input| {
        pg::from(Staff::table())
            .join(Store::table())
            .on(|employee, store| employee.store_id().eq(store.store_id()))
            .join(Staff::table())
            .on(|(_, store), manager| store.manager_staff_id().eq(manager.staff_id()))
            .filter(|((employee, _), _)| employee.store_id().eq(input.store_id))
            .map(|((employee, _), manager)| StaffWithManager {
                staff_id: employee.staff_id,
                first_name: employee.first_name,
                last_name: employee.last_name,
                manager_id: manager.staff_id,
                manager_first_name: manager.first_name,
                manager_last_name: manager.last_name,
            })
            .order_by(|staff| staff.staff_id().asc())
            .limit(50)
    })
    .compile()
}
