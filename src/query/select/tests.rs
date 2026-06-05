use crate::{
    Column, Projection, Table,
    expr::{ExprFragment, Ident},
    value::Value,
};

use super::PgQueryBuilder;

#[test]
fn query_builder_builds_select() {
    let query = PgQueryBuilder
        .select(|q| q.from(foo::table()).project(|foo| (foo.id(), foo.name())))
        .build();

    assert_eq!(query.sql, "SELECT t0.id, t0.name FROM foo AS t0");
    assert_eq!(query.params, []);
}

#[test]
fn query_builder_builds_join_and_filters() {
    let query = PgQueryBuilder
        .select(|q| {
            q.from(foo::table())
                .join(other::table(), |(foo, other)| foo.id().eq(other.id()))
                .filter(|(foo, _)| [foo.id().eq(42), foo.name().like("foo%")])
                .project(|(foo, other)| (foo.id(), other.name()))
        })
        .build();

    assert_eq!(
        query.sql,
        "SELECT t0.id, t1.name FROM foo AS t0 JOIN other AS t1 ON t0.id = t1.id WHERE t0.id = $1 AND t0.name LIKE $2"
    );
    assert_eq!(
        query.params,
        [Value::Int(42), Value::Text("foo%".to_owned())]
    );
}

#[test]
fn query_builder_builds_multiple_joins_with_stable_aliases() {
    let query = PgQueryBuilder
        .select(|q| {
            q.from(foo::table())
                .join(other::table(), |(foo, first)| foo.id().eq(first.id()))
                .join(other::table(), |((foo, _), second)| {
                    foo.id().eq(second.id())
                })
                .filter(|((_, first), second)| {
                    [first.name().like("left%"), second.name().like("right%")]
                })
                .project(|((foo, first), second)| (foo.id(), first.name(), second.name()))
        })
        .build();

    assert_eq!(
        query.sql,
        "SELECT t0.id, t1.name, t2.name FROM foo AS t0 JOIN other AS t1 ON t0.id = t1.id JOIN other AS t2 ON t0.id = t2.id WHERE t1.name LIKE $1 AND t2.name LIKE $2"
    );
    assert_eq!(
        query.params,
        [
            Value::Text("left%".to_owned()),
            Value::Text("right%".to_owned())
        ]
    );
}

mod foo {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Foo;

    #[derive(Clone)]
    pub struct Ref {
        alias: Ident,
    }

    pub fn table() -> Foo {
        Foo
    }

    impl Table for Foo {
        type Row = (i32, String);
        type Ref = Ref;

        const NAME: &'static str = "foo";

        fn bind(alias: Ident) -> Self::Ref {
            Ref { alias }
        }
    }

    impl Ref {
        pub fn id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "id")
        }

        pub fn name(&self) -> Column<String> {
            Column::new(self.alias.clone(), "name")
        }
    }

    impl Projection for Ref {
        type Row = (i32, String);
        type Output = (i32, String);

        fn columns(&self) -> Vec<ExprFragment> {
            (self.id(), self.name()).columns()
        }

        fn bind(self, row: Self::Row) -> Self::Output {
            row
        }
    }
}

mod other {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Other;

    #[derive(Clone)]
    pub struct Ref {
        alias: Ident,
    }

    pub fn table() -> Other {
        Other
    }

    impl Table for Other {
        type Row = (i32, String);
        type Ref = Ref;

        const NAME: &'static str = "other";

        fn bind(alias: Ident) -> Self::Ref {
            Ref { alias }
        }
    }

    impl Ref {
        pub fn id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "id")
        }

        pub fn name(&self) -> Column<String> {
            Column::new(self.alias.clone(), "name")
        }
    }

    impl Projection for Ref {
        type Row = (i32, String);
        type Output = (i32, String);

        fn columns(&self) -> Vec<ExprFragment> {
            (self.id(), self.name()).columns()
        }

        fn bind(self, row: Self::Row) -> Self::Output {
            row
        }
    }
}
