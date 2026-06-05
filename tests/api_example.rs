#[path = "api_example/model.rs"]
mod model;

use placeholder_query::{Fetch, PgQueryBuilder, value::Value};

use model::{PostWithAuthorById, UserById, UserCardById, users};

#[test]
fn table_can_be_projected_as_row_model() {
    let query = PgQueryBuilder.select(|q| q.from(users::table())).build();

    assert_eq!(
        query.sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0"
    );
    assert_eq!(query.params, []);
}

#[test]
fn table_key_builds_whole_row_query() {
    let queries = Fetch::new(|cx| cx.get(UserById { id: 7 })).to_queries(&PgQueryBuilder);

    assert_eq!(
        queries[0].query.sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(queries[0].query.params, vec![Value::Int(7)]);
}

#[test]
fn projection_key_builds_projection_query() {
    let queries = Fetch::new(|cx| cx.get(UserCardById { id: 7 })).to_queries(&PgQueryBuilder);

    assert_eq!(queries.len(), 1);
    assert_eq!(
        queries[0].query.sql,
        "SELECT t0.id, t0.name FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(queries[0].query.params, vec![Value::Int(7)]);
}

#[test]
fn duplicate_table_keys_are_merged_across_fetch_combinators() {
    let queries = Fetch::new(|cx| {
        cx.get(UserById { id: 7 })
            .zip(Fetch::traverse([8, 7, 8], |id| cx.get(UserById { id })))
    })
    .to_queries(&PgQueryBuilder);

    assert_eq!(queries.len(), 1);
    assert_eq!(
        queries[0].query.sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1, $2)"
    );
    assert_eq!(queries[0].query.params, vec![Value::Int(7), Value::Int(8)]);
}

#[test]
fn table_key_and_projection_key_are_distinct_requests() {
    let queries = Fetch::new(|cx| {
        cx.get(UserById { id: 7 })
            .zip(cx.get(UserCardById { id: 7 }))
    })
    .to_queries(&PgQueryBuilder);

    assert_eq!(queries.len(), 2);
    assert_eq!(
        queries[0].query.sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0 WHERE t0.id IN ($1)"
    );
    assert_eq!(
        queries[1].query.sql,
        "SELECT t0.id, t0.name FROM users AS t0 WHERE t0.id IN ($1)"
    );
}

#[test]
fn projection_key_can_join_different_models() {
    let queries =
        Fetch::new(|cx| cx.get(PostWithAuthorById { id: 99 })).to_queries(&PgQueryBuilder);

    assert_eq!(queries.len(), 1);
    assert_eq!(
        queries[0].query.sql,
        "SELECT t0.id, t0.title, t1.name FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id WHERE t0.id IN ($1)"
    );
    assert_eq!(queries[0].query.params, vec![Value::Int(99)]);
}
