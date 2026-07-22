use placeholder_query_postgres::{
    Column, Expr, Ident, Pg, PgFetchBatch, PgFetchKey, Projection, ProjectionExt, Table,
};
use placeholder_query_runtime::FetchKey;
use placeholder_query_tokio_postgres::Executor;
use tokio_postgres::Row;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
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

impl PgFetchKey<Executor> for UserById {
    fn batch(keys: &[Self]) -> impl Into<PgFetchBatch<Executor, Self>> {
        Pg.batch(keys)
            .select(|q, keys| {
                q.from(users::table())
                    .filter(|user| user.id().in_(keys.iter().map(|key| key.id)))
            })
            .keyed_by(|user| UserById { id: user.id })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UserCard {
    pub id: i32,
    pub display_name: String,
}

impl UserCard {
    pub fn project(user: users::Columns) -> impl Projection<Output = Self> {
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

impl PgFetchKey<Executor> for UserCardById {
    fn batch(keys: &[Self]) -> impl Into<PgFetchBatch<Executor, Self>> {
        Pg.batch(keys)
            .select(|q, keys| {
                q.from(users::table())
                    .filter(|user| user.id().in_(keys.iter().map(|key| key.id)))
                    .project(UserCard::project)
            })
            .keyed_by(|card| UserCardById { id: card.id })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Post {
    pub id: i32,
    pub author_id: i32,
    pub title: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostWithAuthor {
    pub post_id: i32,
    pub title: String,
    pub author_name: String,
}

impl PostWithAuthor {
    pub fn project(
        (post, author): (posts::Columns, users::Columns),
    ) -> impl Projection<Output = Self> {
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

impl PgFetchKey<Executor> for PostWithAuthorById {
    fn batch(keys: &[Self]) -> impl Into<PgFetchBatch<Executor, Self>> {
        Pg.batch(keys)
            .select(|q, keys| {
                q.from(posts::table())
                    .join(users::table(), |(post, author)| {
                        post.author_id().eq(author.id())
                    })
                    .filter(|(post, _)| post.id().in_(keys.iter().map(|key| key.id)))
                    .project(PostWithAuthor::project)
            })
            .keyed_by(|post| PostWithAuthorById { id: post.post_id })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PostComment {
    pub id: i32,
    pub post_id: i32,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PostCommentsByPostId {
    pub post_id: i32,
}

impl FetchKey for PostCommentsByPostId {
    type Output = Vec<PostComment>;
}

impl PgFetchKey<Executor> for PostCommentsByPostId {
    fn batch(keys: &[Self]) -> impl Into<PgFetchBatch<Executor, Self>> {
        Pg.batch(keys)
            .select(|q, keys| {
                q.from(post_comments::table())
                    .filter(|comment| comment.post_id().in_(keys.iter().map(|key| key.post_id)))
            })
            .keyed_by(|comment| PostCommentsByPostId {
                post_id: comment.post_id,
            })
    }
}

impl TryFrom<Row> for User {
    type Error = tokio_postgres::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            email: row.try_get("email")?,
        })
    }
}

impl TryFrom<Row> for UserCard {
    type Error = tokio_postgres::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            display_name: row.try_get("name")?,
        })
    }
}

impl TryFrom<Row> for PostWithAuthor {
    type Error = tokio_postgres::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            post_id: row.try_get("id")?,
            title: row.try_get("title")?,
            author_name: row.try_get("name")?,
        })
    }
}

impl TryFrom<Row> for PostComment {
    type Error = tokio_postgres::Error;

    fn try_from(row: Row) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.try_get("id")?,
            post_id: row.try_get("post_id")?,
            body: row.try_get("body")?,
        })
    }
}

pub mod users {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Users;

    #[derive(Clone)]
    pub struct Columns {
        alias: Ident,
    }

    pub fn table() -> Users {
        Users
    }

    impl Table for Users {
        type Row = User;
        type Columns = Columns;

        const NAME: &'static str = "users";

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

        pub fn email(&self) -> Column<String> {
            Column::new(self.alias.clone(), "email")
        }
    }

    impl Projection for Columns {
        type Fields = (i32, String, String);
        type Output = User;

        fn select_exprs(&self) -> Vec<Expr> {
            (self.id(), self.name(), self.email()).select_exprs()
        }

        fn from_fields(&self, fields: Self::Fields) -> Self::Output {
            let (id, name, email) = fields;

            User { id, name, email }
        }
    }
}

pub mod posts {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct Posts;

    #[derive(Clone)]
    pub struct Columns {
        alias: Ident,
    }

    pub fn table() -> Posts {
        Posts
    }

    impl Table for Posts {
        type Row = Post;
        type Columns = Columns;

        const NAME: &'static str = "posts";

        fn bind_alias(alias: Ident) -> Self::Columns {
            Columns { alias }
        }
    }

    impl Columns {
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

    impl Projection for Columns {
        type Fields = (i32, i32, String);
        type Output = Post;

        fn select_exprs(&self) -> Vec<Expr> {
            (self.id(), self.author_id(), self.title()).select_exprs()
        }

        fn from_fields(&self, fields: Self::Fields) -> Self::Output {
            let (id, author_id, title) = fields;

            Post {
                id,
                author_id,
                title,
            }
        }
    }
}

pub mod post_comments {
    use super::*;

    #[derive(Clone, Copy)]
    pub struct PostComments;

    #[derive(Clone)]
    pub struct Columns {
        alias: Ident,
    }

    pub fn table() -> PostComments {
        PostComments
    }

    impl Table for PostComments {
        type Row = PostComment;
        type Columns = Columns;

        const NAME: &'static str = "post_comments";

        fn bind_alias(alias: Ident) -> Self::Columns {
            Columns { alias }
        }
    }

    impl Columns {
        pub fn id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "id")
        }

        pub fn post_id(&self) -> Column<i32> {
            Column::new(self.alias.clone(), "post_id")
        }

        pub fn body(&self) -> Column<String> {
            Column::new(self.alias.clone(), "body")
        }
    }

    impl Projection for Columns {
        type Fields = (i32, i32, String);
        type Output = PostComment;

        fn select_exprs(&self) -> Vec<Expr> {
            (self.id(), self.post_id(), self.body()).select_exprs()
        }

        fn from_fields(&self, fields: Self::Fields) -> Self::Output {
            let (id, post_id, body) = fields;

            PostComment { id, post_id, body }
        }
    }
}
