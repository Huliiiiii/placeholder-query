use chrono::{DateTime, Utc};
use placeholder_query_macro::row;
use placeholder_query_postgres::query::{
    Expr, OptionalRow, mode, select as pg, select::template::SelectTemplate,
};

use super::{CustomerFilter, StoreFilter};
use crate::schema::{Customer, Film, Inventory, Rental};

#[row(derive(Clone, Debug))]
pub struct StockItem {
    pub inventory_id: i32,
    pub film_id: i32,
    pub title: String,
    pub store_id: i32,
}

#[row(derive(Clone, Debug))]
pub struct RentalHistory {
    pub rental_id: i32,
    pub film_id: i32,
    pub title: String,
    pub rental_date: DateTime<Utc>,
    pub return_date: Option<DateTime<Utc>>,
}

pub fn inventory_for_store() -> SelectTemplate<StockItem<mode::Expr>, StoreFilter> {
    pg::query::<StoreFilter, _>(|input| {
        pg::from(Inventory::table())
            .join(Film::table())
            .on(|inventory, film| inventory.film_id().eq(film.film_id()))
            .filter(|(inventory, _)| inventory.store_id().eq(input.store_id))
            .map(|(inventory, film)| StockItem {
                inventory_id: inventory.inventory_id,
                film_id: film.film_id,
                title: film.title,
                store_id: inventory.store_id,
            })
            .order_by(|item| item.title().asc())
            .order_by(|item| item.inventory_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn customer_rental_history() -> SelectTemplate<RentalHistory<mode::Expr>, CustomerFilter> {
    pg::query::<CustomerFilter, _>(|input| {
        pg::from(Rental::table())
            .join(Inventory::table())
            .on(|rental, inventory| rental.inventory_id().eq(inventory.inventory_id()))
            .join(Film::table())
            .on(|(_, inventory), film| inventory.film_id().eq(film.film_id()))
            .filter(|((rental, _), _)| rental.customer_id().eq(input.customer_id))
            .map(|((rental, _), film)| RentalHistory {
                rental_id: rental.rental_id,
                film_id: film.film_id,
                title: film.title,
                rental_date: rental.rental_date,
                return_date: rental.return_date,
            })
            .order_by(|rental| rental.rental_date().desc())
            .order_by(|rental| rental.rental_id().desc())
            .limit(50)
    })
    .compile()
}

#[row(derive(Clone, Debug))]
pub struct RecentRentalCustomer {
    pub rental_id: i32,
    pub customer_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub rental_date: DateTime<Utc>,
}

pub fn recent_rental_customers() -> SelectTemplate<RecentRentalCustomer<mode::Expr>> {
    let recent = pg::from(Rental::table())
        .order_by(|rental| rental.rental_date().desc())
        .order_by(|rental| rental.rental_id().desc())
        .limit(50)
        .map(|rental| (rental.rental_id, rental.customer_id, rental.rental_date));

    pg::from(recent)
        .join(Customer::table())
        .on(|(_, customer_id, _), customer| customer_id.clone().eq(customer.customer_id()))
        .map(
            |((rental_id, customer_id, rental_date), customer)| RecentRentalCustomer {
                rental_id,
                customer_id,
                first_name: customer.first_name,
                last_name: customer.last_name,
                rental_date,
            },
        )
        .order_by(|rental| rental.rental_date().desc())
        .order_by(|rental| rental.rental_id().desc())
        .compile()
}

pub fn customers_with_latest_rental() -> SelectTemplate<
    (
        Expr<i32>,
        Expr<String>,
        Expr<String>,
        OptionalRow<Expr<i32>>,
    ),
    StoreFilter,
> {
    pg::query::<StoreFilter, _>(|input| {
        pg::from(Customer::table())
            .filter(|customer| customer.store_id().eq(input.store_id))
            .left_join_lateral(|customer| {
                pg::from(Rental::table())
                    .filter(|rental| rental.customer_id().eq(customer.customer_id()))
                    .order_by(|rental| rental.rental_date().desc())
                    .order_by(|rental| rental.rental_id().desc())
                    .limit(1)
                    .map(|rental| rental.rental_id)
            })
            .on_true()
            .order_by(|(customer, _)| customer.customer_id().asc())
            .map(|(customer, rental)| {
                (
                    customer.customer_id,
                    customer.first_name,
                    customer.last_name,
                    rental,
                )
            })
            .limit(50)
    })
    .compile()
}
