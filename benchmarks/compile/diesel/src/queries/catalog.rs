use diesel::{
    pg::Pg,
    prelude::*,
    query_builder::{Query, QueryFragment},
};

use diesel::sql_types::{Bpchar, Integer, Nullable, Numeric, Text};

use crate::schema::{actor, category, film, film_actor, film_category, inventory, language};

pub fn films_by_language(
    language_id: i32,
) -> impl Query<SqlType = film::SqlType> + QueryFragment<Pg> {
    film::table
        .filter(film::language_id.eq(language_id))
        .select(film::all_columns)
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn films_with_language() -> impl Query<SqlType = (Integer, Text, Bpchar)> + QueryFragment<Pg> {
    film::table
        .inner_join(language::table.on(film::language_id.eq(language::language_id)))
        .select((film::film_id, film::title, language::name))
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn films_by_actor(actor_id: i32) -> impl Query<SqlType = (Integer, Text)> + QueryFragment<Pg> {
    film::table
        .inner_join(film_actor::table.on(film::film_id.eq(film_actor::film_id)))
        .filter(film_actor::actor_id.eq(actor_id))
        .select((film::film_id, film::title))
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn films_by_category(
    category_id: i32,
) -> impl Query<SqlType = (Integer, Text)> + QueryFragment<Pg> {
    film::table
        .inner_join(film_category::table.on(film::film_id.eq(film_category::film_id)))
        .filter(film_category::category_id.eq(category_id))
        .select((film::film_id, film::title))
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn cast_for_film(
    film_id: i32,
) -> impl Query<SqlType = (Integer, Text, Text)> + QueryFragment<Pg> {
    film_actor::table
        .inner_join(actor::table.on(film_actor::actor_id.eq(actor::actor_id)))
        .filter(film_actor::film_id.eq(film_id))
        .select((actor::actor_id, actor::first_name, actor::last_name))
        .order((actor::last_name, actor::first_name, actor::actor_id))
}

pub fn film_catalog() -> impl Query<
    SqlType = (
        Integer,
        Text,
        Nullable<Text>,
        Numeric,
        Bpchar,
        Integer,
        Text,
    ),
> + QueryFragment<Pg> {
    film::table
        .inner_join(language::table.on(film::language_id.eq(language::language_id)))
        .inner_join(film_category::table.on(film::film_id.eq(film_category::film_id)))
        .inner_join(category::table.on(film_category::category_id.eq(category::category_id)))
        .select((
            film::film_id,
            film::title,
            film::description,
            film::rental_rate,
            language::name,
            category::category_id,
            category::name,
        ))
        .order((film::title, film::film_id, category::category_id))
        .limit(50)
}

pub fn search_films(
    title_pattern: String,
) -> impl Query<SqlType = (Integer, Text)> + QueryFragment<Pg> {
    film::table
        .filter(film::title.like(title_pattern))
        .select((film::film_id, film::title))
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn films_by_ids(
    film_ids: Vec<i32>,
) -> impl Query<SqlType = (Integer, Text)> + QueryFragment<Pg> {
    film::table
        .filter(film::film_id.eq_any(film_ids))
        .select((film::film_id, film::title))
        .order(film::film_id)
}

pub fn filter_films(
    language_id: i32,
    min_rental_duration: i16,
    max_rental_duration: i16,
    title_pattern: String,
    alternative_title_pattern: String,
) -> impl Query<SqlType = (Integer, Text)> + QueryFragment<Pg> {
    film::table
        .filter(
            film::language_id
                .eq(language_id)
                .and(film::rental_duration.ge(min_rental_duration))
                .and(film::rental_duration.le(max_rental_duration))
                .and(
                    film::title
                        .like(title_pattern)
                        .or(film::title.like(alternative_title_pattern)),
                ),
        )
        .select((film::film_id, film::title))
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn films_after(
    title: String,
    film_id: i32,
) -> impl Query<SqlType = (Integer, Text)> + QueryFragment<Pg> {
    film::table
        .filter(
            film::title
                .gt(title.clone())
                .or(film::title.eq(title).and(film::film_id.gt(film_id))),
        )
        .select((film::film_id, film::title))
        .order((film::title, film::film_id))
        .limit(50)
}

pub fn films_with_store_inventory(
    store_id: i32,
) -> impl Query<SqlType = (Integer, Text, Nullable<Integer>)> + QueryFragment<Pg> {
    film::table
        .left_join(
            inventory::table.on(film::film_id
                .eq(inventory::film_id)
                .and(inventory::store_id.eq(store_id))),
        )
        .select((
            film::film_id,
            film::title,
            inventory::inventory_id.nullable(),
        ))
        .order((film::film_id, inventory::inventory_id))
        .limit(50)
}
