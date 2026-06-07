use std::{convert::TryFrom, error::Error, fmt};

use placeholder_query::{ExecutedBatch, Fetch};
use placeholder_query_builder::value::Value;
use placeholder_query_postgres::{PgBackend, PgQuery};
use postgres::{Client, NoTls, types::ToSql};

pub type PgDriverBackend = PgBackend<postgres::Row, postgres::Error>;

pub struct PgExecutor {
    client: Client,
}

impl PgExecutor {
    pub fn connect(params: impl AsRef<str>) -> Result<Self, postgres::Error> {
        Client::connect(params.as_ref(), NoTls).map(Self::from_client)
    }

    pub fn from_client(client: Client) -> Self {
        Self { client }
    }

    pub fn client_mut(&mut self) -> &mut Client {
        &mut self.client
    }

    pub fn execute(
        &mut self,
        fetch: &Fetch<PgDriverBackend>,
    ) -> Result<Vec<ExecutedBatch>, PgExecutorError> {
        fetch.execute(&PgDriverBackend::new(), |query| self.query(query))
    }

    fn query(&mut self, query: &PgQuery) -> Result<Vec<postgres::Row>, PgExecutorError> {
        let params = query
            .params
            .iter()
            .map(PgParam::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let params = params
            .iter()
            .map(PgParam::as_sql)
            .collect::<Vec<&(dyn ToSql + Sync)>>();

        self.client
            .query(&query.sql, &params)
            .map_err(PgExecutorError::from)
    }
}

#[derive(Debug)]
pub enum PgExecutorError {
    Postgres(postgres::Error),
    UnsignedOutOfRange { value: u64 },
}

impl fmt::Display for PgExecutorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Postgres(error) => write!(f, "{error}"),
            Self::UnsignedOutOfRange { value } => {
                write!(
                    f,
                    "unsigned value {value} cannot fit into a postgres signed integer"
                )
            }
        }
    }
}

impl Error for PgExecutorError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Postgres(error) => Some(error),
            Self::UnsignedOutOfRange { .. } => None,
        }
    }
}

impl From<postgres::Error> for PgExecutorError {
    fn from(value: postgres::Error) -> Self {
        Self::Postgres(value)
    }
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

impl TryFrom<&Value> for PgParam {
    type Error = PgExecutorError;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::TinyInt(value) => Ok(Self::I16((*value).into())),
            Value::SmallInt(value) => Ok(Self::I16(*value)),
            Value::Int(value) => Ok(Self::I32(*value)),
            Value::BigInt(value) => Ok(Self::I64(*value)),
            Value::TinyUnsigned(value) => Ok(Self::I16((*value).into())),
            Value::SmallUnsigned(value) => Ok(Self::I32((*value).into())),
            Value::Unsigned(value) => Ok(Self::I64((*value).into())),
            Value::BigUnsigned(value) => {
                let value = i64::try_from(*value)
                    .map_err(|_| PgExecutorError::UnsignedOutOfRange { value: *value })?;

                Ok(Self::I64(value))
            }
            Value::Float(value) => Ok(Self::F32(*value)),
            Value::Double(value) => Ok(Self::F64(*value)),
            Value::Text(value) => Ok(Self::Text(value.clone())),
            Value::Bytes(value) => Ok(Self::Bytes(value.clone())),
        }
    }
}
