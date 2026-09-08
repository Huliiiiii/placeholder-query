use crate::query::{any, select as pg};

use super::{Filters, Foo};

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
