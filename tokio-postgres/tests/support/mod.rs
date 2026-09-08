pub mod schema;

use std::env;

use placeholder_query_tokio_postgres::{Error, Executor};
use tokio_postgres::NoTls;

pub async fn seeded_executor() -> Result<Executor, Error> {
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

INSERT INTO users (id, name, email) VALUES
    (1, 'Ada', 'ada@example.test'),
    (2, 'Ben', 'ben@example.test'),
    (3, 'Cyd', 'cyd@example.test'),
    (7, 'Mio', 'mio@example.test');
"#,
        )
        .await?;

    Ok(executor)
}
