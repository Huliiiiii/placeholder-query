use std::future::Future;

use indexmap::IndexMap;
use placeholder_query_postgres::{PgBackend, PgFetchKey, PgStatement, Value};
use placeholder_query_runtime::{DataSource, Fetch, FetchEnv};
use tokio_postgres::{Client, Row, types::ToSql};

pub struct Executor {
    client: Client,
}

impl Executor {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn batch_execute(&self, sql: impl AsRef<str>) -> Result<(), tokio_postgres::Error> {
        self.client.batch_execute(sql.as_ref()).await
    }

    pub async fn run<A>(&self, fetch: Fetch<Self, A>) -> Result<A, tokio_postgres::Error> {
        fetch.run(self).await
    }
}

impl FetchEnv for Executor {
    type Error = tokio_postgres::Error;
}

impl PgBackend for Executor {
    type Row = Row;
}

impl<K> DataSource<K> for Executor
where
    K: PgFetchKey<Self>,
{
    fn batch_fetch<'a>(
        &'a self,
        keys: &'a [K],
    ) -> impl Future<Output = Result<IndexMap<K, K::Output>, Self::Error>> + 'a {
        async move {
            let batch = K::batch(keys).into();
            let rows = execute_statement(&self.client, batch.statement()).await?;

            batch.collect(rows)
        }
    }
}

async fn execute_statement(
    client: &Client,
    statement: &PgStatement,
) -> Result<Vec<Row>, tokio_postgres::Error> {
    let params = statement
        .params
        .iter()
        .map(|value| -> &(dyn ToSql + Sync) {
            match value {
                Value::SmallInt(value) => value,
                Value::Int(value) => value,
                Value::BigInt(value) => value,
                Value::Real(value) => value,
                Value::Double(value) => value,
                Value::Text(value) => value,
                Value::Bytea(value) => value,
            }
        })
        .collect::<Vec<_>>();

    client.query(&statement.sql, &params).await
}
