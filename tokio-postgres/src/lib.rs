use std::{convert::TryFrom, future::Future};

use futures_util::future::try_join_all;
use placeholder_query_core::value::Value;
use placeholder_query_postgres::{PgFetchBackend, PgStatement};
use placeholder_query_runtime::{AsyncFetchExecutor, Fetch};
use tokio_postgres::{Client, Row, types::ToSql};

pub type FetchBackend = PgFetchBackend<Row, tokio_postgres::Error>;

pub struct Executor {
    client: Client,
}

impl Executor {
    pub async fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn batch_execute(
        &mut self,
        sql: impl AsRef<str>,
    ) -> Result<(), tokio_postgres::Error> {
        self.client.batch_execute(sql.as_ref()).await
    }

    pub async fn run<A>(
        &mut self,
        fetch: Fetch<FetchBackend, A>,
    ) -> Result<A, tokio_postgres::Error> {
        fetch.run_async(self).await
    }
}

impl AsyncFetchExecutor<FetchBackend> for Executor {
    type Error = tokio_postgres::Error;

    fn execute_round(
        &mut self,
        statements: Vec<PgStatement>,
    ) -> impl Future<Output = Result<Vec<Vec<Row>>, Self::Error>> + '_ {
        execute_round_postgres(&self.client, statements)
    }
}

async fn execute_round_postgres(
    client: &Client,
    statements: Vec<PgStatement>,
) -> Result<Vec<Vec<Row>>, tokio_postgres::Error> {
    try_join_all(
        statements
            .into_iter()
            .map(|statement| execute_statement(client, statement)),
    )
    .await
}

async fn execute_statement(
    client: &Client,
    statement: PgStatement,
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
