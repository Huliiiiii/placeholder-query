use crate::query::select as pg;
use rel9_macro::QueryInput;

use super::User;

#[test]
#[should_panic(expected = "column refers to a source outside this query")]
fn ordinary_join_inputs_have_disjoint_scopes() {
    let mut outer_id = None;
    let outer = pg::from(User::table()).map(|user| {
        outer_id = Some(user.id());
        user
    });
    let inner = pg::from(User::table()).filter(|user| user.id().eq(outer_id.unwrap()));

    outer.cross_join(inner).compile();
}

#[derive(QueryInput)]
struct UserEmailFilter {
    email: String,
}

#[test]
fn reused_join_sources_keep_outer_scope() {
    let users = pg::from(User::table());
    let query = pg::query::<UserEmailFilter, _>(|input| {
        users
            .clone()
            .filter(|user| user.email().eq(input.email))
            .cross_join_lateral(|outer| {
                let mut candidate = None;
                let candidates = users.map(|candidate_row| {
                    let id = candidate_row.id();
                    candidate = Some(candidate_row);
                    id
                });
                let candidate = candidate.unwrap();

                candidates
                    .right_join(User::table())
                    .on(|id, manager| id.clone().gt(manager.id()))
                    .filter(|_| candidate.id().gt(outer.id()))
                    .order_by(|_| candidate.id().asc())
                    .limit(1)
                    .clone()
            })
    });
    let template = query.compile();
    let statement = template.statement();

    assert_sql!(
        statement.sql(),
        r#"
        SELECT t0."id", t0."name", t0."email", t1.c0 IS NOT NULL, t1.c1, t1.c2, t1.c3, t1.c4
        FROM "users" AS t0
        CROSS JOIN LATERAL (
            SELECT t2.c0, t2.c1, t3."id", t3."name", t3."email"
            FROM (
                SELECT TRUE, t4."id", t4."name", t4."email"
                FROM "users" AS t4
            ) AS t2(c0, c1, c2, c3)
            RIGHT JOIN "users" AS t3
            ON t2.c1 > t3."id"
            WHERE t2.c1 > t0."id"
            ORDER BY t2.c1
            LIMIT 1
        ) AS t1(c0, c1, c2, c3, c4)
        WHERE t0."email" = $1
        "#
    );
}
