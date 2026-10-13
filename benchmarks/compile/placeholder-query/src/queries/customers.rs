use placeholder_query_macro::row;
use placeholder_query_postgres::query::{mode, select as pg, select::template::SelectTemplate};

use super::{StoreCustomerFilter, StoreFilter};
use crate::schema::{Address, City, Country, Customer};

#[row(derive(Clone, Debug))]
pub struct CustomerContact {
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: Option<String>,
    pub address: String,
    pub address2: Option<String>,
    pub district: String,
    pub postal_code: Option<String>,
    pub city: String,
    pub country: String,
    pub phone: String,
}

pub fn customers_by_store() -> SelectTemplate<Customer<mode::Expr>, StoreCustomerFilter> {
    pg::query::<StoreCustomerFilter, _>(|input| {
        pg::from(Customer::table())
            .filter(|customer| {
                customer
                    .store_id()
                    .eq(input.store_id)
                    .and(customer.activebool().eq(input.active))
            })
            .order_by(|customer| customer.last_name().asc())
            .order_by(|customer| customer.first_name().asc())
            .order_by(|customer| customer.customer_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn customer_contacts() -> SelectTemplate<CustomerContact<mode::Expr>, StoreFilter> {
    pg::query::<StoreFilter, _>(|input| {
        pg::from(Customer::table())
            .join(Address::table())
            .on(|customer, address| customer.address_id().eq(address.address_id()))
            .join(City::table())
            .on(|(_, address), city| address.city_id().eq(city.city_id()))
            .join(Country::table())
            .on(|(_, city), country| city.country_id().eq(country.country_id()))
            .filter(|(((customer, _), _), _)| customer.store_id().eq(input.store_id))
            .map(|(((customer, address), city), country)| CustomerContact {
                customer_id: customer.customer_id,
                first_name: customer.first_name,
                last_name: customer.last_name,
                email: customer.email,
                address: address.address,
                address2: address.address2,
                district: address.district,
                postal_code: address.postal_code,
                city: city.city,
                country: country.country,
                phone: address.phone,
            })
            .order_by(|customer| customer.customer_id().asc())
            .limit(50)
    })
    .compile()
}
