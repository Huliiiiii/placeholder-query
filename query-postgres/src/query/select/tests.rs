use crate::{Column, Expr, Pg, Projection, Table, TableAlias, Value};

#[test]
fn empty_in_filter_renders_false() {
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
fn in_filter_binds_each_value() {
    let statement = Pg
        .select(|q| {
            q.from(foo::table())
                .filter(|foo| foo.id().in_([1, 2]))
                .project(|foo| foo.id())
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id FROM foo AS t0 WHERE t0.id IN ($1, $2)"
    );
    assert_eq!(statement.params, [Value::Int(1), Value::Int(2)]);
}

#[test]
fn composed_filter_preserves_grouping() {
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
fn five_column_projection_renders_all_columns() {
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

    #[derive(Clone, Copy)]
    pub struct Columns {
        alias: TableAlias,
    }

    pub fn table() -> Foo {
        Foo
    }

    impl Table for Foo {
        type Row = (i32, String);
        type Columns = Columns;

        const NAME: &'static str = "foo";

        fn bind_alias(alias: TableAlias) -> Self::Columns {
            Columns { alias }
        }
    }

    impl Columns {
        pub fn id(&self) -> Column<i32> {
            Column::new(self.alias, "id")
        }

        pub fn name(&self) -> Column<String> {
            Column::new(self.alias, "name")
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
