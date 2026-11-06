use rel9_macro::QueryInput;

#[derive(QueryInput)]
pub(super) struct Filters {
    pub(super) id: i32,
    pub(super) ids: Vec<i32>,
    pub(super) pattern: String,
}

#[rel9_macro::table]
pub(super) struct Foo {
    pub(super) id: i32,
    pub(super) name: String,
}

#[rel9_macro::table(name = "users")]
pub(super) struct User {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) email: String,
}

macro_rules! assert_sql {
    ($actual:expr, $expected:literal $(,)?) => {{
        let expected = $expected
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .fold(String::new(), |mut sql, line| {
                if !sql.is_empty() && !sql.ends_with('(') && !line.starts_with(')') {
                    sql.push(' ');
                }
                sql.push_str(line);
                sql
            });

        assert_eq!($actual, expected);
    }};
}

mod composition;
mod correlation;
mod parameters;
mod rendering;
