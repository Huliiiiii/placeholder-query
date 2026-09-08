use crate::query::projection::ColumnValue;
use crate::query::{Projection, ProjectionExt, mode, select as pg, table::TableSchema};
use placeholder_query_macro::{QueryInput, row};

use super::{Filters, Foo, User};

#[test]
fn derived_tables_keep_row_shape() {
    let inner = pg::from(Foo::table()).map(|foo| {
        (Foo {
            id: foo.id,
            name: foo.name,
        },)
    });
    let middle = pg::from(inner).map(|(foo,)| (foo.name, foo.id));
    let template = pg::from(middle).map(|(_, id)| id).compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0.c1 FROM (SELECT t1.c1, t1.c0 FROM (SELECT t2."id", t2."name" FROM "foo" AS t2) AS t1(c0, c1)) AS t0(c0, c1)"#
    );
}

#[test]
fn lateral_tables_are_first_class_rows() {
    let query = pg::query::<Filters, _>(|params| {
        pg::from(Foo::table())
            .lateral(|outer| {
                pg::from(Foo::table())
                    .filter(|inner| inner.id().eq(outer.id))
                    .order_by(|inner| inner.id().desc())
                    .limit(1)
                    .map(|inner| inner.id)
            })
            .filter(|id| id.clone().gt(params.id))
            .map(|id| id.eq(params.id))
    });

    assert_eq!(
        query.compile().statement().sql(),
        r#"SELECT t1.c0 = $1 FROM "foo" AS t0 CROSS JOIN LATERAL (SELECT t2."id" FROM "foo" AS t2 WHERE t2."id" = t0."id" ORDER BY t2."id" DESC LIMIT 1) AS t1(c0) WHERE t1.c0 > $1"#
    );
}

#[derive(QueryInput)]
struct UserEmailFilter {
    email: String,
}

#[row(derive(Clone, Debug, PartialEq, Eq))]
struct Contact {
    name: String,
    email: String,
}

#[test]
fn array_inputs_behave_like_tables() {
    #[derive(QueryInput)]
    struct UserLookup {
        id: i32,
        name: String,
    }

    let query = pg::query::<[UserLookup], _>(|inputs| {
        pg::from(inputs.unnest())
            .join(User::table(), |(input, user)| {
                input.id().eq(user.id()).and(input.name().eq(user.name()))
            })
            .map(|(_, user)| user)
    })
    .compile();

    assert_eq!(
        query.statement().sql(),
        r#"SELECT t1."id", t1."name", t1."email" FROM unnest($1::integer[], $2::text[]) AS t0("id", "name") JOIN "users" AS t1 ON (t0."id" = t1."id" AND t0."name" = t1."name")"#
    );
}

#[test]
fn named_rows_keep_field_identity() {
    let table = TableSchema::new(
        "users",
        (
            mode::Name::<i32>::new("id"),
            (
                (),
                Contact::<mode::Name> {
                    name: mode::Name::new("email"),
                    email: mode::Name::new("name"),
                },
            ),
        ),
    );
    let pairs = pg::from(table.clone())
        .join(table, |((user_id, _), (manager_id, _))| {
            manager_id.clone().gt(user_id.clone())
        })
        .order_by(|((user_id, _), _)| user_id.clone().asc())
        .order_by(|(_, (manager_id, _))| manager_id.clone().asc())
        .limit(1)
        .map(|((user_id, (unit, user)), (manager_id, (_, manager)))| {
            (
                Contact {
                    name: user.email,
                    email: user.name,
                },
                (manager.name, user_id, manager_id, unit),
            )
        });
    let query = pg::query::<UserEmailFilter, _>(|input| {
        pg::from(pairs)
            .filter(|(_, (manager_email, _, _, _))| manager_email.clone().eq(input.email))
            .map(|(contact, (manager_email, user_id, manager_id, unit))| {
                ((manager_id, user_id, unit), contact, manager_email)
            })
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0.c4, t0.c3, t0.c0, t0.c1, t0.c2 FROM (SELECT t1."name", t1."email", t2."email", t1."id", t2."id" FROM "users" AS t1 JOIN "users" AS t2 ON t2."id" > t1."id" ORDER BY t1."id", t2."id" LIMIT 1) AS t0(c0, c1, c2, c3, c4) WHERE t0.c2 = $1"#
    );
}

#[test]
fn mapped_outputs_decode_from_source_fields() {
    let prefix = "Contact: ".to_owned();
    let query = pg::query::<UserEmailFilter, _>(|input| {
        let contacts = pg::from(User::table())
            .filter(|user| user.email().eq(input.email))
            .map(|user| {
                (user.name, user.email).map(|(name, email)| format!("{prefix}{name} <{email}>"))
            });
        pg::from(pg::from(contacts)).clone()
    });
    let template = query.compile();

    assert_eq!(
        template.statement().sql(),
        r#"SELECT t0.c0, t0.c1 FROM (SELECT t1.c0, t1.c1 FROM (SELECT t2."name", t2."email" FROM "users" AS t2 WHERE t2."email" = $1) AS t1(c0, c1)) AS t0(c0, c1)"#
    );
    assert_eq!(
        template.projection().from_fields((
            ColumnValue("Ada".to_owned()),
            ColumnValue("ada@example.test".to_owned()),
        )),
        "Contact: Ada <ada@example.test>"
    );
}

#[test]
fn reused_selects_get_fresh_source_scopes() {
    let users: pg::Select<User<mode::Expr>> = pg::from(User::table());
    let query = pg::query::<UserEmailFilter, _>(|input| {
        let pairs = pg::from(users.clone())
            .join(users.clone(), |(user, manager)| manager.id().gt(user.id()))
            .filter(|(user, _)| user.email().eq(input.email))
            .order_by(|(_, manager)| manager.id().asc())
            .limit(1);

        pg::from(pairs).map(|(user, manager)| (user.id, user.name, manager.id, manager.name))
    });
    let template = query.compile();
    let statement = template.statement();

    assert_eq!(
        statement.sql(),
        r#"SELECT t0.c0, t0.c1, t0.c3, t0.c4 FROM (SELECT t1.c0, t1.c1, t1.c2, t2.c0, t2.c1, t2.c2 FROM (SELECT t3."id", t3."name", t3."email" FROM "users" AS t3) AS t1(c0, c1, c2) JOIN (SELECT t4."id", t4."name", t4."email" FROM "users" AS t4) AS t2(c0, c1, c2) ON t2.c0 > t1.c0 WHERE t1.c2 = $1 ORDER BY t2.c0 LIMIT 1) AS t0(c0, c1, c2, c3, c4, c5)"#
    );
}
