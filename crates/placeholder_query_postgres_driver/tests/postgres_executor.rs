#[path = "api_example/model.rs"]
mod model;

use std::error::Error;

use placeholder_query::Fetch;
use placeholder_query_postgres_driver::PgExecutor;

use model::{PostWithAuthor, PostWithAuthorById, User, UserById};

const DEFAULT_DATABASE_URL: &str =
    "postgres://placeholder_query:placeholder_query@localhost:55432/placeholder_query";

#[test]
#[ignore = "requires docker compose postgres"]
fn executor_runs_fetch_batches_against_postgres() -> Result<(), Box<dyn Error>> {
    let url = std::env::var("PLACEHOLDER_QUERY_TEST_DATABASE_URL")
        .unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_owned());
    let mut executor = PgExecutor::connect(url)?;

    executor.client_mut().batch_execute(
        "
        DROP TABLE IF EXISTS posts;
        DROP TABLE IF EXISTS users;

        CREATE TABLE users (
            id integer PRIMARY KEY,
            name text NOT NULL,
            email text NOT NULL
        );

        CREATE TABLE posts (
            id integer PRIMARY KEY,
            author_id integer NOT NULL REFERENCES users(id),
            title text NOT NULL
        );

        INSERT INTO users (id, name, email) VALUES
            (1, 'Mio', 'mio@example.test'),
            (2, 'Ritsu', 'ritsu@example.test');

        INSERT INTO posts (id, author_id, title) VALUES
            (10, 1, 'Typed queries');
        ",
    )?;

    let fetch = Fetch::new(|cx| {
        cx.get(UserById { id: 1 })
            .zip(cx.get(UserById { id: 2 }))
            .zip(cx.get(UserById { id: 1 }))
            .zip(cx.get(PostWithAuthorById { id: 10 }))
    });
    let batches = executor.execute(&fetch)?;

    assert_eq!(batches.len(), 2);

    let users = batches[0].outputs::<UserById>().unwrap();
    assert_eq!(batches[0].key_count, 2);
    assert_eq!(
        users,
        &[
            Some(User {
                id: 1,
                name: "Mio".to_owned(),
                email: "mio@example.test".to_owned(),
            }),
            Some(User {
                id: 2,
                name: "Ritsu".to_owned(),
                email: "ritsu@example.test".to_owned(),
            }),
        ]
    );

    let posts = batches[1].outputs::<PostWithAuthorById>().unwrap();
    assert_eq!(batches[1].key_count, 1);
    assert_eq!(
        posts,
        &[Some(PostWithAuthor {
            post_id: 10,
            title: "Typed queries".to_owned(),
            author_name: "Mio".to_owned(),
        })]
    );

    Ok(())
}
