use crate::{Column, Expr, Ident, Pg, Projection, Table, Value};

#[test]
fn pg_builds_empty_in_filter() {
    let statement = Pg
        .select(|q| {
            q.from(foo::table())
                .filter(|foo| foo.id().in_(std::iter::empty::<i32>()))
                .project(|foo| foo.id())
        })
        .build();

    assert_eq!(statement.sql, "SELECT t0.id FROM foo AS t0 WHERE FALSE");
    assert_eq!(statement.params, []);
}

#[test]
fn pg_builds_composed_filter() {
    let statement = Pg
        .select(|q| {
            q.from(foo::table())
                .filter(|foo| foo.id().eq(1).or(foo.id().eq(2)).and(foo.name().like("A%")))
                .project(|foo| foo.id())
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id FROM foo AS t0 WHERE ((t0.id = $1 OR t0.id = $2) AND t0.name LIKE $3)"
    );
    assert_eq!(
        statement.params,
        [Value::Int(1), Value::Int(2), Value::Text("A%".to_owned())]
    );
}

#[test]
fn pg_builds_five_column_projection() {
    let statement = Pg
        .select(|q| {
            q.from(foo::table())
                .project(|foo| (foo.id(), foo.name(), foo.id(), foo.name(), foo.id()))
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t0.name, t0.id, t0.name, t0.id FROM foo AS t0"
    );
    assert_eq!(statement.params, []);
}

mod foo {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Foo;

    #[derive(Clone)]
    pub struct Columns {
        alias: Ident,
    }

    pub fn table() -> Foo {
        Foo
    }

    impl Table for Foo {
        type Row = (i32, String);
        type Columns = Columns;

        const NAME: &'static str = "foo";

        fn bind_alias(alias: Ident) -> Self::Columns {
            Columns { alias }
        }
    }

    impl Columns {
        pub fn id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "id")
        }

        pub fn name(&self) -> Column<String> {
            Column::new(self.alias.clone(), "name")
        }
    }

    impl Projection for Columns {
        type Fields = (i32, String);
        type Output = (i32, String);

        fn select_exprs(&self) -> Vec<Expr> {
            (self.id(), self.name()).select_exprs()
        }

        fn from_fields(&self, fields: Self::Fields) -> Self::Output {
            fields
        }
    }
}
