pub mod decode;
pub mod params;
mod statement_cache;

use std::{collections::HashMap, fmt, future::Future, num::NonZeroUsize, pin::pin};

use futures_util::{Stream, StreamExt, TryStreamExt};
use placeholder_query_postgres::{
    Projection,
    query::{
        params::{
            QueryInput,
            record::{ArrayFields, InputRecord},
        },
        select::{BoundSelect, Select, template::SelectTemplate},
    },
};
use placeholder_query_postgres_fetch::{CardinalityError, FromRows};
use placeholder_query_runtime::{Fetch, FetchEnv, FetchError, Request};
use tokio_postgres::Client;

use decode::{DecodeRow, RowDecoder};
use params::BindParams;
use statement_cache::StatementCache;

#[derive(Debug)]
pub enum Error {
    Executor(tokio_postgres::Error),
    Cardinality(CardinalityError),
}

pub type ExecError = FetchError<Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Executor(error) => error.fmt(f),
            Self::Cardinality(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Executor(error) => Some(error),
            Self::Cardinality(error) => Some(error),
        }
    }
}

impl From<tokio_postgres::Error> for Error {
    fn from(error: tokio_postgres::Error) -> Self {
        Self::Executor(error)
    }
}

impl From<CardinalityError> for Error {
    fn from(error: CardinalityError) -> Self {
        Self::Cardinality(error)
    }
}

pub struct Executor {
    client: Client,
    statement_cache: StatementCache,
}

impl Executor {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            statement_cache: StatementCache::new(NonZeroUsize::new(100).unwrap()),
        }
    }

    /// Sets the maximum number of cached statements on this connection.
    ///
    /// The default capacity is 100.
    /// Eviction preserves statements that are still held by running queries.
    pub fn statement_cache_capacity(mut self, capacity: NonZeroUsize) -> Self {
        self.statement_cache.resize(capacity);
        self
    }

    /// Clears the prepared statement cache for this connection.
    pub fn clear_statement_cache(&self) {
        self.statement_cache.clear();
    }

    /// Pass sql to the internal [`tokio_postgres::Client::batch_execute`]
    pub async fn batch_execute(&self, sql: impl AsRef<str>) -> Result<(), Error> {
        self.client.batch_execute(sql.as_ref()).await?;

        Ok(())
    }

    /// Executes a batch query template.
    ///
    /// `reqs` must be unique.
    pub async fn fetch_batch<'input, R, V>(
        &self,
        query: &SelectTemplate<impl Projection<Output = (R, V), Fields: DecodeRow>, [R]>,
        reqs: impl IntoIterator<Item = &'input R>,
    ) -> Result<HashMap<R, R::Output>, Error>
    where
        R: Request + InputRecord + Clone + 'input,
        R::Output: FromRows<V>,
        R::Fields: ArrayFields,
        <R::Fields as ArrayFields>::Arrays<'input>: BindParams,
    {
        let reqs = reqs.into_iter();
        let mut rows_by_req = HashMap::<_, Vec<V>>::with_capacity(reqs.size_hint().0);

        let values = R::Fields::transpose(reqs.map(|req| {
            rows_by_req.insert(req.clone(), Vec::new());
            req.fields()
        }));

        let mut row_stream = pin!(self.query(query, &values).await?);
        while let Some((req, value)) = row_stream.try_next().await? {
            if let Some(rows) = rows_by_req.get_mut(&req) {
                rows.push(value);
            }
        }

        rows_by_req
            .into_iter()
            .map(|(req, rows)| Ok((req, R::Output::from_rows(rows)?)))
            .collect()
    }

    /// Executes a query or fetch computation.
    pub async fn run<R>(&self, computation: R) -> Result<R::Output, ExecError>
    where
        R: Executable,
    {
        computation.run_on(self).await
    }

    async fn query<'a, P, Input: ?Sized>(
        &self,
        query: &'a SelectTemplate<P, Input>,
        bound_values: &impl BindParams,
    ) -> Result<impl Stream<Item = Result<<P as Projection>::Output, Error>> + 'a, Error>
    where
        P: Projection,
        P::Fields: DecodeRow,
    {
        let statement = query.statement();
        let input_values = bound_values.params().collect::<Vec<_>>();
        let parameters = statement
            .params()
            .iter()
            .map(|param| input_values[param.index]);

        let prepared = self
            .statement_cache
            .prepare(&self.client, statement)
            .await?;

        let rows = self
            .client
            .query_raw(&prepared, parameters)
            .await?
            .map_err(Error::from);

        let projection = query.projection();

        Ok(
            rows.map(move |row| -> Result<<P as Projection>::Output, Error> {
                let row = row?;
                let fields = P::Fields::decode(&mut RowDecoder::new(&row)).map_err(Error::from)?;

                Ok(projection.from_fields(fields))
            }),
        )
    }
}

#[doc(hidden)]
pub trait Executable {
    type Output;

    fn run_on(self, executor: &Executor) -> impl Future<Output = Result<Self::Output, ExecError>>;
}

impl<A> Executable for Fetch<Executor, A>
where
    A: 'static,
{
    type Output = A;

    async fn run_on(self, executor: &Executor) -> Result<Self::Output, ExecError> {
        FetchEnv::run(executor, self).await
    }
}

impl<P> Executable for Select<P, ()>
where
    P: Projection,
    P::Fields: DecodeRow,
{
    type Output = Vec<<P as Projection>::Output>;

    async fn run_on(self, executor: &Executor) -> Result<Self::Output, ExecError> {
        self.compile().run_on(executor).await
    }
}

impl<P> Executable for SelectTemplate<P, ()>
where
    P: Projection,
    P::Fields: DecodeRow,
{
    type Output = Vec<<P as Projection>::Output>;

    async fn run_on(self, executor: &Executor) -> Result<Self::Output, ExecError> {
        BoundSelect {
            select: self,
            params: &(),
        }
        .run_on(executor)
        .await
    }
}

impl<'input, P, Input: ?Sized> Executable for BoundSelect<'input, P, Input>
where
    P: Projection,
    Input: QueryInput,
    Input::Values<'input>: BindParams,
    P::Fields: DecodeRow,
{
    type Output = Vec<<P as Projection>::Output>;

    async fn run_on(self, executor: &Executor) -> Result<Self::Output, ExecError> {
        let BoundSelect { select, params } = self;
        let values = params.values();

        Ok(executor
            .query(&select, &values)
            .await?
            .try_collect()
            .await?)
    }
}

impl FetchEnv for Executor {
    type Error = Error;
}
