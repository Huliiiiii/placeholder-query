use placeholder_query_postgres::query::{mode, select as pg, select::template::SelectTemplate};

use crate::schema::*;

pub fn all_actors() -> SelectTemplate<Actor<mode::Expr>> {
    pg::from(Actor::table()).compile()
}

pub fn all_films() -> SelectTemplate<Film<mode::Expr>> {
    pg::from(Film::table()).compile()
}

pub fn all_languages() -> SelectTemplate<Language<mode::Expr>> {
    pg::from(Language::table()).compile()
}

pub fn all_film_actors() -> SelectTemplate<FilmActor<mode::Expr>> {
    pg::from(FilmActor::table()).compile()
}

pub fn all_categories() -> SelectTemplate<Category<mode::Expr>> {
    pg::from(Category::table()).compile()
}

pub fn all_film_categories() -> SelectTemplate<FilmCategory<mode::Expr>> {
    pg::from(FilmCategory::table()).compile()
}

pub fn all_addresses() -> SelectTemplate<Address<mode::Expr>> {
    pg::from(Address::table()).compile()
}

pub fn all_cities() -> SelectTemplate<City<mode::Expr>> {
    pg::from(City::table()).compile()
}

pub fn all_countries() -> SelectTemplate<Country<mode::Expr>> {
    pg::from(Country::table()).compile()
}

pub fn all_customers() -> SelectTemplate<Customer<mode::Expr>> {
    pg::from(Customer::table()).compile()
}

pub fn all_inventory() -> SelectTemplate<Inventory<mode::Expr>> {
    pg::from(Inventory::table()).compile()
}

pub fn all_rentals() -> SelectTemplate<Rental<mode::Expr>> {
    pg::from(Rental::table()).compile()
}

pub fn all_payments() -> SelectTemplate<Payment<mode::Expr>> {
    pg::from(Payment::table()).compile()
}

pub fn all_staff() -> SelectTemplate<Staff<mode::Expr>> {
    pg::from(Staff::table()).compile()
}

pub fn all_stores() -> SelectTemplate<Store<mode::Expr>> {
    pg::from(Store::table()).compile()
}
