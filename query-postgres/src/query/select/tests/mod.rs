use placeholder_query_macro::QueryInput;

#[derive(QueryInput)]
pub(super) struct Filters {
    pub(super) id: i32,
    pub(super) ids: Vec<i32>,
    pub(super) pattern: String,
}

#[placeholder_query_macro::table]
pub(super) struct Foo {
    pub(super) id: i32,
    pub(super) name: String,
}

#[placeholder_query_macro::table(name = "users")]
pub(super) struct User {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) email: String,
}

mod composition;
mod correlation;
mod parameters;
mod rendering;
