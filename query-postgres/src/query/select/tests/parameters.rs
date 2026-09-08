use crate::query::params::SqlType;
use crate::query::{Expr, any, select as pg};

use super::{Filters, Foo};

#[test]
fn array_inputs_bind_as_one_parameter() {
    let query = pg::query::<Filters, _>(|params| {
        pg::from(Foo::table())
            .filter(|foo| foo.id().eq(any(params.ids)))
            .map(|foo| foo.id)
    });
    let template = query.compile();
    let statement = template.statement();
    let params = statement.params();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0."id" FROM "foo" AS t0 WHERE t0."id" = ANY($1)"#
    );
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].index, 1);
    assert_eq!(params[0].ty, SqlType::Array(Box::new(SqlType::Int4)));
}

#[test]
fn one_input_field_gets_one_placeholder() {
    let query = pg::query::<Filters, _>(|params| {
        pg::from(Foo::table())
            .filter(|foo| foo.id().eq(params.id).or(foo.id().eq(params.id)))
            .map(|foo| foo.id)
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0."id" FROM "foo" AS t0 WHERE (t0."id" = $1 OR t0."id" = $1)"#
    );
}

#[test]
fn placeholders_follow_render_order() {
    let query = pg::query::<Filters, _>(|params| {
        let names = pg::from(Foo::table())
            .filter(|foo| foo.id().eq(params.id))
            .map(|foo| foo.name);

        pg::from(names).map(|name| name.like(params.pattern))
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0.c0 LIKE $1 FROM (SELECT t1."name" FROM "foo" AS t1 WHERE t1."id" = $2) AS t0(c0)"#
    );
    assert_eq!(
        statement
            .params()
            .iter()
            .map(|param| (param.index, param.ty.clone()))
            .collect::<Vec<_>>(),
        [(2, SqlType::Text), (0, SqlType::Int4)]
    );
}

#[test]
#[should_panic(expected = "parameter refers to a different query input scope")]
fn input_fields_are_scoped_to_their_query() {
    let mut captured = None;
    let _first = pg::query::<Filters, _>(|params| {
        captured = Some(Expr::from(params.id));
        pg::from(Foo::table()).map(|foo| foo.id)
    });

    let captured = captured.expect("the first query stores its parameter");
    pg::query::<Filters, _>(|_| {
        pg::from(Foo::table())
            .filter(|foo| foo.id().eq(captured))
            .map(|foo| foo.id)
    })
    .compile();
}
