use placeholder_query_core::{
    column::Column,
    expr::{Expr, Ident},
    projection::{Projection, ProjectionExt},
    table::Table,
};
use placeholder_query_runtime::{FetchBatch, FetchKey};

use crate::backend::{TestPostgresBackend, TestRow};

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

impl FetchKey<TestPostgresBackend> for UserById {
    type Output = Option<User>;

    fn batch(keys: &[Self]) -> impl Into<FetchBatch<TestPostgresBackend, Self>> {
        TestPostgresBackend::batch(keys)
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

impl FetchKey<TestPostgresBackend> for UserCardById {
    type Output = Option<UserCard>;

    fn batch(keys: &[Self]) -> impl Into<FetchBatch<TestPostgresBackend, Self>> {
        TestPostgresBackend::batch(keys)
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

impl FetchKey<TestPostgresBackend> for PostWithAuthorById {
    type Output = Option<PostWithAuthor>;

    fn batch(keys: &[Self]) -> impl Into<FetchBatch<TestPostgresBackend, Self>> {
        TestPostgresBackend::batch(keys)
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

impl FetchKey<TestPostgresBackend> for PostCommentsByPostId {
    type Output = Vec<PostComment>;

    fn batch(keys: &[Self]) -> impl Into<FetchBatch<TestPostgresBackend, Self>> {
        TestPostgresBackend::batch(keys)
            .select(|q, keys| {
                q.from(post_comments::table())
                    .filter(|comment| comment.post_id().in_(keys.iter().map(|key| key.post_id)))
            })
            .keyed_by(|comment| PostCommentsByPostId {
                post_id: comment.post_id,
            })
    }
}

impl TryFrom<TestRow> for User {
    type Error = std::convert::Infallible;

    fn try_from(row: TestRow) -> Result<Self, Self::Error> {
        let TestRow::User(user) = row else {
            panic!("expected user row in api example test")
        };

        Ok(user)
    }
}

impl TryFrom<TestRow> for UserCard {
    type Error = std::convert::Infallible;

    fn try_from(row: TestRow) -> Result<Self, Self::Error> {
        let TestRow::User(user) = row else {
            panic!("expected user row for user card in api example test")
        };

        Ok(UserCard {
            id: user.id,
            display_name: user.name,
        })
    }
}

impl TryFrom<TestRow> for PostWithAuthor {
    type Error = std::convert::Infallible;

    fn try_from(row: TestRow) -> Result<Self, Self::Error> {
        let TestRow::PostWithAuthor(post) = row else {
            panic!("expected post-with-author row in api example test")
        };

        Ok(post)
    }
}

impl TryFrom<TestRow> for PostComment {
    type Error = std::convert::Infallible;

    fn try_from(row: TestRow) -> Result<Self, Self::Error> {
        let TestRow::PostComment(comment) = row else {
            panic!("expected post comment row in api example test")
        };

        Ok(comment)
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
