use crate::query::{any, select as pg};

use super::{Filters, Foo, User};

#[test]
fn identifiers_are_escaped() {
    use crate::query::{mode, table::TableSchema};

    let query = pg::from(TableSchema::new(
        r#"user"archive"#,
        mode::Name::<String>::new(r#"display"name"#),
    ))
    .compile();

    assert_eq!(
        query.statement().sql(),
        r#"SELECT t0."display""name" FROM "user""archive" AS t0"#
    );
}

#[test]
fn boolean_connectives_are_grouped() {
    let query = pg::query::<Filters, _>(|params| {
        pg::from(Foo::table())
            .filter(|foo| {
                foo.id()
                    .eq(params.id)
                    .or(foo.id().eq(any(params.ids)))
                    .and(foo.name().like(params.pattern))
            })
            .map(|foo| foo.id)
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0."id" FROM "foo" AS t0 WHERE ((t0."id" = $1 OR t0."id" = ANY($2)) AND t0."name" LIKE $3)"#
    );
}

#[test]
fn comparison_operands_are_grouped() {
    let query = pg::query::<Filters, _>(|params| {
        pg::from(Foo::table()).map(|foo| foo.id.eq(params.id).eq(foo.name.like(params.pattern)))
    });

    assert_eq!(
        query.compile().statement().sql(),
        r#"SELECT (t0."id" = $1) = (t0."name" LIKE $2) FROM "foo" AS t0"#
    );
}

#[test]
fn full_join_preserves_inputs_and_maybe_tags() {
    let query = pg::query::<Filters, _>(|params| {
        let left = pg::from(Foo::table());
        let right = pg::from(Foo::table())
            .filter(|foo| foo.name().like(params.pattern))
            .limit(2);

        left.full_join(right)
            .on(|left, right| left.id().eq(right.id()))
            .map(|(left, right)| (left.map(|foo| foo.name), right.map(|foo| foo.name)))
    });

    assert_sql!(
        query.compile().statement().sql(),
        r#"
        SELECT t0.c0 IS NOT NULL, t0.c2, t1.c0 IS NOT NULL, t1.c2
        FROM (
            SELECT TRUE, t2."id", t2."name"
            FROM "foo" AS t2
        ) AS t0(c0, c1, c2)
        FULL JOIN (
            SELECT TRUE, t3.c0, t3.c1
            FROM (
                SELECT t4."id", t4."name"
                FROM "foo" AS t4
                WHERE t4."name" LIKE $1
                LIMIT 2
            ) AS t3(c0, c1)
        ) AS t1(c0, c1, c2)
        ON t0.c1 = t1.c1
        "#
    );
}

#[test]
fn filters_can_reuse_columns_across_outer_joins() {
    let query = pg::query::<Filters, _>(|params| {
        let mut original = None;
        let users = pg::from(User::table())
            .filter(|user| user.email().eq(params.pattern))
            .map(|user| {
                let id = user.id();
                original = Some(user);
                id
            });
        let user = original.unwrap();

        users
            .full_join(Foo::table())
            .on(|_, foo| user.id().eq(foo.id()))
            .right_join(User::table())
            .on(|_, other| user.id().eq(other.id()))
            .order_by(|_| user.name().asc())
            .map(|(_, other)| other.id)
    });

    assert_sql!(
        query.compile().statement().sql(),
        r#"
        SELECT t1."id"
        FROM (
            SELECT TRUE, t2.c0, t2.c1, t3.c0, t3.c1, t3.c2, t2.c2, t2.c3
            FROM (
                SELECT TRUE, t4."id", t4."name", t4."email"
                FROM "users" AS t4
            ) AS t2(c0, c1, c2, c3)
            FULL JOIN (
                SELECT TRUE, t5."id", t5."name"
                FROM "foo" AS t5
            ) AS t3(c0, c1, c2)
            ON t2.c1 = t3.c1
        ) AS t0(c0, c1, c2, c3, c4, c5, c6, c7)
        RIGHT JOIN "users" AS t1 ON t0.c2 = t1."id"
        WHERE t0.c7 = $1
        ORDER BY t0.c6
        "#
    );
}
