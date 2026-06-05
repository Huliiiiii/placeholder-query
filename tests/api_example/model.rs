use placeholder_query::{
    Batch, FetchKey, PgQueryBuilder,
    column::Column,
    expr::{ExprFragment, Ident},
    projection::{Projection, ProjectionExt},
    query::select::PgQueryPlan,
    table::Table,
};

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserById {
    pub id: i32,
}

impl FetchKey for UserById {
    type Output = Option<User>;
}

impl Batch<UserById> for PgQueryBuilder {
    fn plan(&self, keys: &[UserById]) -> PgQueryPlan {
        self.select(|q| {
            q.from(users::table())
                .filter(|user| user.id().in_(keys.iter().map(|key| key.id)))
        })
        .into()
    }

    fn collect(
        &self,
        keys: &[UserById],
        rows: Vec<postgres::Row>,
    ) -> Result<Vec<Option<User>>, postgres::Error> {
        let mut users = HashMap::new();
        for row in rows {
            let user = User {
                id: row.try_get(0)?,
                name: row.try_get(1)?,
                email: row.try_get(2)?,
            };
            users.insert(user.id, user);
        }

        Ok(keys.iter().map(|key| users.remove(&key.id)).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserCard {
    pub id: i32,
    pub display_name: String,
}

impl UserCard {
    pub fn project(user: users::Ref) -> impl Projection<Row = (i32, String), Output = Self> {
        (user.id(), user.name()).map(|(id, display_name)| Self { id, display_name })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UserCardById {
    pub id: i32,
}

impl FetchKey for UserCardById {
    type Output = Option<UserCard>;
}

impl Batch<UserCardById> for PgQueryBuilder {
    fn plan(&self, keys: &[UserCardById]) -> PgQueryPlan {
        self.select(|q| {
            q.from(users::table())
                .filter(|user| user.id().in_(keys.iter().map(|key| key.id)))
                .project(UserCard::project)
        })
        .into()
    }

    fn collect(
        &self,
        keys: &[UserCardById],
        rows: Vec<postgres::Row>,
    ) -> Result<Vec<Option<UserCard>>, postgres::Error> {
        let mut users = HashMap::new();
        for row in rows {
            let user = UserCard {
                id: row.try_get(0)?,
                display_name: row.try_get(1)?,
            };
            users.insert(user.id, user);
        }

        Ok(keys.iter().map(|key| users.remove(&key.id)).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Post {
    pub id: i32,
    pub author_id: i32,
    pub title: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PostWithAuthor {
    pub post_id: i32,
    pub title: String,
    pub author_name: String,
}

impl PostWithAuthor {
    pub fn project(
        (post, author): (posts::Ref, users::Ref),
    ) -> impl Projection<Row = (i32, String, String), Output = Self> {
        (post.id(), post.title(), author.name()).map(|(post_id, title, author_name)| Self {
            post_id,
            title,
            author_name,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostWithAuthorById {
    pub id: i32,
}

impl FetchKey for PostWithAuthorById {
    type Output = Option<PostWithAuthor>;
}

impl Batch<PostWithAuthorById> for PgQueryBuilder {
    fn plan(&self, keys: &[PostWithAuthorById]) -> PgQueryPlan {
        self.select(|q| {
            q.from(posts::table())
                .join(users::table(), |(post, author)| {
                    post.author_id().eq(author.id())
                })
                .filter(|(post, _)| post.id().in_(keys.iter().map(|key| key.id)))
                .project(PostWithAuthor::project)
        })
        .into()
    }

    fn collect(
        &self,
        keys: &[PostWithAuthorById],
        rows: Vec<postgres::Row>,
    ) -> Result<Vec<Option<PostWithAuthor>>, postgres::Error> {
        let mut posts = HashMap::new();
        for row in rows {
            let post = PostWithAuthor {
                post_id: row.try_get(0)?,
                title: row.try_get(1)?,
                author_name: row.try_get(2)?,
            };
            posts.insert(post.post_id, post);
        }

        Ok(keys.iter().map(|key| posts.remove(&key.id)).collect())
    }
}

pub mod users {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Users;

    #[derive(Clone)]
    pub struct Ref {
        alias: Ident,
    }

    pub fn table() -> Users {
        Users
    }

    impl Table for Users {
        type Row = User;
        type Ref = Ref;

        const NAME: &'static str = "users";

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

        pub fn email(&self) -> Column<String> {
            Column::new(self.alias.clone(), "email")
        }
    }

    impl Projection for Ref {
        type Row = (i32, String, String);
        type Output = User;

        fn columns(&self) -> Vec<ExprFragment> {
            (self.id(), self.name(), self.email()).columns()
        }

        fn bind(self, row: Self::Row) -> Self::Output {
            let (id, name, email) = row;

            User { id, name, email }
        }
    }
}

pub mod posts {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Posts;

    #[derive(Clone)]
    pub struct Ref {
        alias: Ident,
    }

    pub fn table() -> Posts {
        Posts
    }

    impl Table for Posts {
        type Row = Post;
        type Ref = Ref;

        const NAME: &'static str = "posts";

        fn bind(alias: Ident) -> Self::Ref {
            Ref { alias }
        }
    }

    impl Ref {
        pub fn id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "id")
        }

        pub fn author_id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "author_id")
        }

        pub fn title(&self) -> Column<String> {
            Column::new(self.alias.clone(), "title")
        }
    }

    impl Projection for Ref {
        type Row = (i32, i32, String);
        type Output = Post;

        fn columns(&self) -> Vec<ExprFragment> {
            (self.id(), self.author_id(), self.title()).columns()
        }

        fn bind(self, row: Self::Row) -> Self::Output {
            let (id, author_id, title) = row;

            Post {
                id,
                author_id,
                title,
            }
        }
    }
}
