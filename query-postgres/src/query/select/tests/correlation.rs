use crate::query::select as pg;
use placeholder_query_macro::QueryInput;

use super::User;

#[test]
#[should_panic(expected = "column refers to a source outside this query")]
fn ordinary_sources_have_disjoint_scopes() {
    let mut outer_id = None;
    let outer = pg::from(User::table()).map(|user| {
        outer_id = Some(user.id());
        user
    });
    let inner = pg::from(User::table()).filter(|user| user.id().eq(outer_id.unwrap()));

    outer
        .join(inner, |(left, right)| left.id().eq(right.id()))
        .compile();
}

#[derive(QueryInput)]
struct UserEmailFilter {
    email: String,
}

#[test]
fn reused_lateral_sources_keep_outer_scope() {
    let users = pg::from(User::table());
    let query = pg::query::<UserEmailFilter, _>(|input| {
        users
            .clone()
            .filter(|user| user.email().eq(input.email))
            .lateral(|user| {
                let candidates = users.filter(|candidate| candidate.id().gt(user.id()));
                candidates
                    .clone()
                    .order_by(|candidate| candidate.id().asc())
                    .limit(1)
            })
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t1.c0, t1.c1, t1.c2 FROM "users" AS t0 CROSS JOIN LATERAL (SELECT t2."id", t2."name", t2."email" FROM "users" AS t2 WHERE t2."id" > t0."id" ORDER BY t2."id" LIMIT 1) AS t1(c0, c1, c2) WHERE t0."email" = $1"#
    );
}

#[test]
fn nested_laterals_keep_each_outer_scope() {
    let query = pg::query::<UserEmailFilter, _>(|input| {
        let rows = pg::from(User::table())
            .filter(|outer| outer.email().eq(input.email))
            .lateral(|outer| {
                pg::from(User::table())
                    .filter(|middle| middle.id().gt(outer.id()))
                    .order_by(|middle| middle.id().asc())
                    .limit(1)
                    .lateral(|middle| {
                        pg::from(User::table())
                            .filter(|inner| inner.id().gt(middle.id()))
                            .order_by(|inner| inner.id().asc())
                            .limit(1)
                            .map(|inner| (outer.id, middle.id, inner.id))
                    })
            });
        pg::from(rows).clone()
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0.c0, t0.c1, t0.c2 FROM (SELECT t2.c0, t2.c1, t2.c2 FROM "users" AS t1 CROSS JOIN LATERAL (SELECT t4.c0, t4.c1, t4.c2 FROM "users" AS t3 CROSS JOIN LATERAL (SELECT t1."id", t3."id", t5."id" FROM "users" AS t5 WHERE t5."id" > t3."id" ORDER BY t5."id" LIMIT 1) AS t4(c0, c1, c2) WHERE t3."id" > t1."id" ORDER BY t3."id" LIMIT 1) AS t2(c0, c1, c2) WHERE t1."email" = $1) AS t0(c0, c1, c2)"#
    );
}
