#[path = "api_example/model.rs"]
mod model;

use std::env;

use placeholder_query_postgres::{Pg, Value};
use placeholder_query_runtime::Fetch;
use placeholder_query_tokio_postgres::Executor;
use tokio_postgres::NoTls;

use model::{
    PostComment, PostCommentsByPostId, PostWithAuthor, PostWithAuthorById, User, UserById,
    UserCard, UserCardById, post_comments, posts, users,
};

async fn seeded_executor() -> Result<Executor, tokio_postgres::Error> {
    let database_url = env::var("PLACEHOLDER_QUERY_DATABASE_URL").unwrap_or_else(|_| {
        "host=localhost port=55432 user=placeholder_query password=placeholder_query dbname=placeholder_query"
            .to_owned()
    });
    let (client, connection) = tokio_postgres::connect(&database_url, NoTls).await?;

    tokio::spawn(async move {
        if let Err(error) = connection.await {
            panic!("postgres connection failed: {error}");
        }
    });

    let executor = Executor::new(client);
    executor
        .batch_execute(
            r#"
CREATE TEMP TABLE users (
    id integer PRIMARY KEY,
    name text NOT NULL,
    email text NOT NULL
);

CREATE TEMP TABLE posts (
    id integer PRIMARY KEY,
    author_id integer NOT NULL REFERENCES users(id),
    title text NOT NULL
);

CREATE TEMP TABLE post_comments (
    id integer PRIMARY KEY,
    post_id integer NOT NULL REFERENCES posts(id),
    body text NOT NULL
);

INSERT INTO users (id, name, email) VALUES
    (1, 'Ada', 'ada@example.test'),
    (2, 'Ben', 'ben@example.test'),
    (3, 'Cyd', 'cyd@example.test'),
    (4, 'Dee', 'dee@example.test'),
    (5, 'Eve', 'eve@example.test'),
    (7, 'Mio', 'mio@example.test'),
    (12, 'Nia', 'nia@example.test'),
    (225, 'Sum', 'sum@example.test');

INSERT INTO posts (id, author_id, title) VALUES
    (10, 1, 'Ten'),
    (11, 1, 'Eleven'),
    (20, 2, 'Twenty'),
    (21, 2, 'Twenty One'),
    (30, 3, 'Thirty'),
    (40, 4, 'Forty'),
    (44, 2, 'Computed'),
    (50, 5, 'Fifty'),
    (99, 7, 'Intro');

INSERT INTO post_comments (id, post_id, body) VALUES
    (1010, 10, 'first on ten'),
    (1011, 11, 'first on eleven'),
    (1020, 20, 'first on twenty'),
    (1021, 21, 'first on twenty one'),
    (1030, 30, 'first on thirty');
"#,
        )
        .await?;

    Ok(executor)
}

#[test]
fn table_projection() {
    let statement = Pg.select(|q| q.from(users::table())).build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t0.name, t0.email FROM users AS t0"
    );
    assert_eq!(statement.params, []);
}

#[test]
fn partial_table_projection() {
    let statement = Pg
        .select(|q| q.from(users::table()).project(UserCard::project))
        .build();

    assert_eq!(statement.sql, "SELECT t0.id, t0.name FROM users AS t0");
    assert_eq!(statement.params, []);
}

#[test]
fn nested_join_projection() {
    let statement = Pg
        .select(|q| {
            q.from(posts::table())
                .join(users::table(), |(post, author)| {
                    post.author_id().eq(author.id())
                })
                .join(post_comments::table(), |((post, _), comment)| {
                    post.id().eq(comment.post_id())
                })
                .filter(|((post, author), comment)| {
                    [
                        post.id().eq(99),
                        author.name().like("M%"),
                        comment.body().like("%first%"),
                    ]
                })
                .project(|((post, author), comment)| (post.id(), author.name(), comment.body()))
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t1.name, t2.body FROM posts AS t0 JOIN users AS t1 ON t0.author_id = t1.id JOIN post_comments AS t2 ON t0.id = t2.post_id WHERE t0.id = $1 AND t1.name LIKE $2 AND t2.body LIKE $3"
    );
    assert_eq!(
        statement.params,
        vec![
            Value::Int(99),
            Value::Text("M%".to_owned()),
            Value::Text("%first%".to_owned()),
        ]
    );
}

#[test]
fn self_join_projection_uses_distinct_aliases() {
    let statement = Pg
        .select(|q| {
            q.from(users::table())
                .join(users::table(), |(user, manager)| user.id().eq(manager.id()))
                .filter(|(_, manager)| [manager.email().like("%@example.test")])
                .project(|(user, manager)| (user.id(), user.name(), manager.email()))
        })
        .build();

    assert_eq!(
        statement.sql,
        "SELECT t0.id, t0.name, t1.email FROM users AS t0 JOIN users AS t1 ON t0.id = t1.id WHERE t1.email LIKE $1"
    );
    assert_eq!(
        statement.params,
        vec![Value::Text("%@example.test".to_owned())]
    );
}

#[tokio::test]
async fn fetch_full_row() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let user = executor
        .run(Fetch::new(|cx| cx.fetch(UserById { id: 7 })))
        .await?;

    assert_eq!(
        user,
        Some(User {
            id: 7,
            name: "Mio".to_owned(),
            email: "mio@example.test".to_owned(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn fetch_projected_row() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let card = executor
        .run(Fetch::new(|cx| cx.fetch(UserCardById { id: 7 })))
        .await?;

    assert_eq!(
        card,
        Some(UserCard {
            id: 7,
            display_name: "Mio".to_owned(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn fetch_join_projection() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let post = executor
        .run(Fetch::new(|cx| cx.fetch(PostWithAuthorById { id: 99 })))
        .await?;

    assert_eq!(
        post,
        Some(PostWithAuthor {
            post_id: 99,
            title: "Intro".to_owned(),
            author_name: "Mio".to_owned(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn fetch_many_rows() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let comments = executor
        .run(Fetch::new(|cx| {
            cx.fetch(PostCommentsByPostId { post_id: 10 })
        }))
        .await?;

    assert_eq!(
        comments,
        vec![PostComment {
            id: 1010,
            post_id: 10,
            body: "first on ten".to_owned(),
        }]
    );

    Ok(())
}

#[tokio::test]
async fn missing_dependent_fetch_stops() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let result = executor
        .run(Fetch::new(|cx| {
            cx.fetch(UserCardById { id: 999 })
                .and_then(|card, cx| match card {
                    Some(card) => cx.fetch(UserById { id: card.id }),
                    None => Fetch::pure(None),
                })
        }))
        .await?;

    assert_eq!(result, None);

    Ok(())
}

#[tokio::test]
async fn deep_dependency_chain_uses_previous_results() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let user = executor
        .run(Fetch::new(|cx| {
            cx.fetch(UserById { id: 1 })
                .and_then(|user, cx| match user {
                    Some(user) => {
                        cx.fetch(PostWithAuthorById { id: user.id + 10 })
                            .and_then(|post, cx| match post {
                                Some(post) => cx
                                    .fetch(PostCommentsByPostId {
                                        post_id: post.post_id,
                                    })
                                    .and_then(move |comments, cx| {
                                        let next_id = comments
                                            .first()
                                            .map(|comment| comment.post_id + 1)
                                            .unwrap_or(post.post_id + 1);

                                        cx.fetch(UserById { id: next_id })
                                    }),
                                None => Fetch::pure(None),
                            })
                    }
                    None => Fetch::pure(None),
                })
        }))
        .await?;

    assert_eq!(
        user,
        Some(User {
            id: 12,
            name: "Nia".to_owned(),
            email: "nia@example.test".to_owned(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn diamond_dependency_joins_again() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let post = executor
        .run(Fetch::new(|cx| {
            cx.fetch(UserById { id: 1 })
                .and_then(|user, cx| match user {
                    Some(user) => cx
                        .fetch(UserById { id: user.id + 1 })
                        .zip(cx.fetch(PostWithAuthorById { id: user.id + 20 }))
                        .zip(cx.fetch(PostCommentsByPostId {
                            post_id: user.id + 20,
                        }))
                        .and_then(|((left, right), comments), cx| match (left, right) {
                            (Some(left), Some(right)) if !comments.is_empty() => {
                                cx.fetch(PostWithAuthorById {
                                    id: left.id + right.post_id + comments[0].post_id,
                                })
                            }
                            _ => Fetch::pure(None),
                        }),
                    None => Fetch::pure(None),
                })
        }))
        .await?;

    assert_eq!(
        post,
        Some(PostWithAuthor {
            post_id: 44,
            title: "Computed".to_owned(),
            author_name: "Ben".to_owned(),
        })
    );

    Ok(())
}

#[tokio::test]
async fn fan_out_fan_in_uses_wide_independent_layer() -> Result<(), tokio_postgres::Error> {
    let executor = seeded_executor().await?;
    let user = executor
        .run(Fetch::new(|cx| {
            cx.traverse(1..=5, |id, cx| cx.fetch(UserById { id }))
                .zip(cx.traverse([10, 20, 30, 40, 50], |id, cx| {
                    cx.fetch(PostWithAuthorById { id })
                }))
                .zip(cx.traverse([10, 20, 30], |post_id, cx| {
                    cx.fetch(PostCommentsByPostId { post_id })
                }))
                .and_then(|((users, posts), comments), cx| {
                    let next_id = users.into_iter().flatten().map(|user| user.id).sum::<i32>()
                        + posts
                            .into_iter()
                            .flatten()
                            .map(|post| post.post_id)
                            .sum::<i32>()
                        + comments
                            .into_iter()
                            .flatten()
                            .map(|comment| comment.post_id)
                            .sum::<i32>();

                    cx.fetch(UserById { id: next_id })
                })
        }))
        .await?;

    assert_eq!(
        user,
        Some(User {
            id: 225,
            name: "Sum".to_owned(),
            email: "sum@example.test".to_owned(),
        })
    );

    Ok(())
}
