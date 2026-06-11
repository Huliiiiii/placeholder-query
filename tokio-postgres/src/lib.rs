use std::{convert::TryFrom, future::Future};

use indexmap::IndexMap;
use placeholder_query_core::value::Value;
use placeholder_query_postgres::{PgBackend, PgFetchKey, PgStatement};
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
        .map(PgParam::from)
        .collect::<Vec<_>>();

    let params = params.iter().map(|p| p.as_sql()).collect::<Vec<_>>();

    client.query(&statement.sql, &params).await
}

enum PgParam {
    I16(i16),
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
    Text(String),
    Bytes(Vec<u8>),
}

impl PgParam {
    fn as_sql(&self) -> &(dyn ToSql + Sync) {
        match self {
            Self::I16(value) => value,
            Self::I32(value) => value,
            Self::I64(value) => value,
            Self::F32(value) => value,
            Self::F64(value) => value,
            Self::Text(value) => value,
            Self::Bytes(value) => value,
        }
    }
}

impl From<&Value> for PgParam {
    fn from(value: &Value) -> Self {
        match value {
            Value::TinyInt(value) => Self::I16((*value).into()),
            Value::SmallInt(value) => Self::I16(*value),
            Value::Int(value) => Self::I32(*value),
            Value::BigInt(value) => Self::I64(*value),
            Value::TinyUnsigned(value) => Self::I16((*value).into()),
            Value::SmallUnsigned(value) => Self::I32((*value).into()),
            Value::Unsigned(value) => Self::I64((*value).into()),
            Value::BigUnsigned(value) => {
                // TODO: pg specific value
                let value = i64::try_from(*value).unwrap();

                Self::I64(value)
            }
            Value::Float(value) => Self::F32(*value),
            Value::Double(value) => Self::F64(*value),
            Value::Text(value) => Self::Text(value.clone()),
            Value::Bytes(value) => Self::Bytes(value.clone()),
        }
    }
}
