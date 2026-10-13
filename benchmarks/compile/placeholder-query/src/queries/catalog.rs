use bigdecimal::BigDecimal;
use placeholder_query_macro::{QueryInput, row};
use placeholder_query_postgres::query::{
    Expr, OptionalRow, any, mode, select as pg, select::template::SelectTemplate,
};

use super::{ActorFilter, CategoryFilter, FilmFilter, LanguageFilter, StoreFilter};
use crate::schema::{Actor, Category, Film, FilmActor, FilmCategory, Inventory, Language};

#[row(derive(Clone, Debug))]
pub struct FilmLanguage {
    pub film_id: i32,
    pub title: String,
    pub language_name: String,
}

#[row(derive(Clone, Debug))]
pub struct FilmListing {
    pub film_id: i32,
    pub title: String,
}

#[row(derive(Clone, Debug))]
pub struct ActorListing {
    pub actor_id: i32,
    pub first_name: String,
    pub last_name: String,
}

#[row(derive(Clone, Debug))]
pub struct CatalogEntry {
    pub film_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub rental_rate: BigDecimal,
    pub language_name: String,
    pub category_id: i32,
    pub category_name: String,
}

pub fn films_by_language() -> SelectTemplate<Film<mode::Expr>, LanguageFilter> {
    pg::query::<LanguageFilter, _>(|input| {
        pg::from(Film::table())
            .filter(|film| film.language_id().eq(input.language_id))
            .order_by(|film| film.title().asc())
            .order_by(|film| film.film_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn films_with_language() -> SelectTemplate<FilmLanguage<mode::Expr>> {
    pg::from(Film::table())
        .join(Language::table())
        .on(|film, language| film.language_id().eq(language.language_id()))
        .map(|(film, language)| FilmLanguage {
            film_id: film.film_id,
            title: film.title,
            language_name: language.name,
        })
        .order_by(|film| film.title().asc())
        .order_by(|film| film.film_id().asc())
        .limit(50)
        .compile()
}

pub fn films_by_actor() -> SelectTemplate<FilmListing<mode::Expr>, ActorFilter> {
    pg::query::<ActorFilter, _>(|input| {
        pg::from(Film::table())
            .join(FilmActor::table())
            .on(|film, cast| film.film_id().eq(cast.film_id()))
            .filter(|(_, cast)| cast.actor_id().eq(input.actor_id))
            .map(|(film, _)| FilmListing {
                film_id: film.film_id,
                title: film.title,
            })
            .order_by(|film| film.title().asc())
            .order_by(|film| film.film_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn films_by_category() -> SelectTemplate<FilmListing<mode::Expr>, CategoryFilter> {
    pg::query::<CategoryFilter, _>(|input| {
        pg::from(Film::table())
            .join(FilmCategory::table())
            .on(|film, category| film.film_id().eq(category.film_id()))
            .filter(|(_, category)| category.category_id().eq(input.category_id))
            .map(|(film, _)| FilmListing {
                film_id: film.film_id,
                title: film.title,
            })
            .order_by(|film| film.title().asc())
            .order_by(|film| film.film_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn cast_for_film() -> SelectTemplate<ActorListing<mode::Expr>, FilmFilter> {
    pg::query::<FilmFilter, _>(|input| {
        pg::from(FilmActor::table())
            .join(Actor::table())
            .on(|cast, actor| cast.actor_id().eq(actor.actor_id()))
            .filter(|(cast, _)| cast.film_id().eq(input.film_id))
            .map(|(_, actor)| ActorListing {
                actor_id: actor.actor_id,
                first_name: actor.first_name,
                last_name: actor.last_name,
            })
            .order_by(|actor| actor.last_name().asc())
            .order_by(|actor| actor.first_name().asc())
            .order_by(|actor| actor.actor_id().asc())
    })
    .compile()
}

pub fn film_catalog() -> SelectTemplate<CatalogEntry<mode::Expr>> {
    pg::from(Film::table())
        .join(Language::table())
        .on(|film, language| film.language_id().eq(language.language_id()))
        .join(FilmCategory::table())
        .on(|(film, _), category| film.film_id().eq(category.film_id()))
        .join(Category::table())
        .on(|(_, film_category), category| film_category.category_id().eq(category.category_id()))
        .map(|(((film, language), _), category)| CatalogEntry {
            film_id: film.film_id,
            title: film.title,
            description: film.description,
            rental_rate: film.rental_rate,
            language_name: language.name,
            category_id: category.category_id,
            category_name: category.name,
        })
        .order_by(|film| film.title().asc())
        .order_by(|film| film.film_id().asc())
        .order_by(|film| film.category_id().asc())
        .limit(50)
        .compile()
}

#[derive(QueryInput)]
pub struct FilmSearch {
    pub title_pattern: String,
}

pub fn search_films() -> SelectTemplate<FilmListing<mode::Expr>, FilmSearch> {
    pg::query::<FilmSearch, _>(|input| {
        pg::from(Film::table())
            .filter(|film| film.title().like(input.title_pattern))
            .map(|film| FilmListing {
                film_id: film.film_id,
                title: film.title,
            })
            .order_by(|film| film.title().asc())
            .order_by(|film| film.film_id().asc())
            .limit(50)
    })
    .compile()
}

#[derive(QueryInput)]
pub struct FilmIds {
    pub film_ids: Vec<i32>,
}

pub fn films_by_ids() -> SelectTemplate<FilmListing<mode::Expr>, FilmIds> {
    pg::query::<FilmIds, _>(|input| {
        pg::from(Film::table())
            .filter(|film| film.film_id().eq(any(input.film_ids)))
            .map(|film| FilmListing {
                film_id: film.film_id,
                title: film.title,
            })
            .order_by(|film| film.film_id().asc())
    })
    .compile()
}

#[derive(QueryInput)]
pub struct FilmFilters {
    pub language_id: i32,
    pub min_rental_duration: i16,
    pub max_rental_duration: i16,
    pub title_pattern: String,
    pub alternative_title_pattern: String,
}

pub fn filter_films() -> SelectTemplate<FilmListing<mode::Expr>, FilmFilters> {
    pg::query::<FilmFilters, _>(|input| {
        pg::from(Film::table())
            .filter(|film| {
                film.language_id()
                    .eq(input.language_id)
                    .and(film.rental_duration().gte(input.min_rental_duration))
                    .and(Expr::from(input.max_rental_duration).gte(film.rental_duration()))
                    .and(
                        film.title()
                            .like(input.title_pattern)
                            .or(film.title().like(input.alternative_title_pattern)),
                    )
            })
            .map(|film| FilmListing {
                film_id: film.film_id,
                title: film.title,
            })
            .order_by(|film| film.title().asc())
            .order_by(|film| film.film_id().asc())
            .limit(50)
    })
    .compile()
}

#[derive(QueryInput)]
pub struct FilmCursor {
    pub title: String,
    pub film_id: i32,
}

pub fn films_after() -> SelectTemplate<FilmListing<mode::Expr>, FilmCursor> {
    pg::query::<FilmCursor, _>(|input| {
        pg::from(Film::table())
            .filter(|film| {
                film.title().gt(input.title).or(film
                    .title()
                    .eq(input.title)
                    .and(film.film_id().gt(input.film_id)))
            })
            .map(|film| FilmListing {
                film_id: film.film_id,
                title: film.title,
            })
            .order_by(|film| film.title().asc())
            .order_by(|film| film.film_id().asc())
            .limit(50)
    })
    .compile()
}

pub fn films_with_store_inventory()
-> SelectTemplate<(FilmListing<mode::Expr>, OptionalRow<Expr<i32>>), StoreFilter> {
    pg::query::<StoreFilter, _>(|input| {
        let mut inventory_id = None;

        let films = pg::from(Film::table())
            .left_join(Inventory::table())
            .on(|film, inventory| {
                inventory_id = Some(inventory.inventory_id());

                film.film_id()
                    .eq(inventory.film_id())
                    .and(inventory.store_id().eq(input.store_id))
            });

        let inventory_id = inventory_id.unwrap();

        films
            .order_by(|(film, _)| film.film_id().asc())
            .order_by(|_| inventory_id.asc())
            .map(|(film, inventory)| {
                (
                    FilmListing {
                        film_id: film.film_id,
                        title: film.title,
                    },
                    inventory.map(|inventory| inventory.inventory_id),
                )
            })
            .limit(50)
    })
    .compile()
}
