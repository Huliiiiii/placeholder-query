use placeholder_query_postgres::PgFetchBackend;

use crate::model::{PostComment, PostWithAuthor, User};

pub type TestPostgresBackend = PgFetchBackend<TestRow, std::convert::Infallible>;

#[derive(Clone, Debug)]
pub enum TestRow {
    User(User),
    PostWithAuthor(PostWithAuthor),
    PostComment(PostComment),
}
