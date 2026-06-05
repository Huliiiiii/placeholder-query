use std::{
    any::{Any, TypeId, type_name},
    collections::HashSet,
    hash::Hash,
};

use postgres::Row;

use crate::query::select::{PgQuery, PgQueryBuilder, PgQueryPlan};

pub trait QueryBuilder {
    type Plan;
    type Query;

    fn compile(&self, plan: &Self::Plan) -> Self::Query;
}

impl QueryBuilder for PgQueryBuilder {
    type Plan = PgQueryPlan;
    type Query = PgQuery;

    fn compile(&self, plan: &Self::Plan) -> Self::Query {
        self.build(plan)
    }
}

pub trait FetchKey: Clone + Eq + Hash + Send + Sync + 'static {
    type Output: Send + Sync + 'static;
}

pub trait Batch<K>: QueryBuilder
where
    K: FetchKey,
{
    fn plan(&self, keys: &[K]) -> Self::Plan;

    fn collect(&self, keys: &[K], rows: Vec<Row>) -> Result<Vec<K::Output>, postgres::Error>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlannedQuery<Query> {
    pub key_type_id: TypeId,
    pub key_type_name: &'static str,
    pub key_count: usize,
    pub query: Query,
}

pub struct ExecutedBatch {
    pub key_type_id: TypeId,
    pub key_type_name: &'static str,
    pub key_count: usize,
    outputs: Box<dyn Any + Send + Sync>,
}

impl ExecutedBatch {
    pub(crate) fn new<K>(outputs: Vec<K::Output>) -> Self
    where
        K: FetchKey,
    {
        Self {
            key_type_id: TypeId::of::<K>(),
            key_type_name: type_name::<K>(),
            key_count: outputs.len(),
            outputs: Box::new(outputs),
        }
    }

    pub fn outputs<K>(&self) -> Option<&[K::Output]>
    where
        K: FetchKey,
    {
        self.outputs
            .downcast_ref::<Vec<K::Output>>()
            .map(Vec::as_slice)
    }
}

pub(crate) trait FetchKeyNode<B>: Send + Sync
where
    B: QueryBuilder,
{
    fn as_any(&self) -> &dyn Any;

    fn key_type_id(&self) -> TypeId;

    fn new_batch(&self) -> Box<dyn BatchNode<B>>;
}

impl<B, K> FetchKeyNode<B> for K
where
    B: Batch<K>,
    K: FetchKey,
{
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn key_type_id(&self) -> TypeId {
        TypeId::of::<K>()
    }

    fn new_batch(&self) -> Box<dyn BatchNode<B>> {
        Box::new(KeyBatch::<K>::new())
    }
}

pub(crate) trait BatchNode<B>: Send
where
    B: QueryBuilder,
{
    fn insert(&mut self, key: &dyn FetchKeyNode<B>);

    fn build_query(&self, builder: &B) -> PlannedQuery<B::Query>;

    fn collect(&self, builder: &B, rows: Vec<Row>) -> Result<ExecutedBatch, postgres::Error>;
}

struct KeyBatch<K>
where
    K: FetchKey,
{
    keys: Vec<K>,
    seen: HashSet<K>,
}

impl<K> KeyBatch<K>
where
    K: FetchKey,
{
    fn new() -> Self {
        Self {
            keys: Vec::new(),
            seen: HashSet::new(),
        }
    }
}

impl<B, K> BatchNode<B> for KeyBatch<K>
where
    B: Batch<K>,
    K: FetchKey,
{
    fn insert(&mut self, key: &dyn FetchKeyNode<B>) {
        let key = key
            .as_any()
            .downcast_ref::<K>()
            .expect("fetch key type should match batch type");

        if self.seen.insert(key.clone()) {
            self.keys.push(key.clone());
        }
    }

    fn build_query(&self, builder: &B) -> PlannedQuery<B::Query> {
        let plan = builder.plan(&self.keys);

        PlannedQuery {
            key_type_id: TypeId::of::<K>(),
            key_type_name: type_name::<K>(),
            key_count: self.keys.len(),
            query: builder.compile(&plan),
        }
    }

    fn collect(&self, builder: &B, rows: Vec<Row>) -> Result<ExecutedBatch, postgres::Error> {
        builder
            .collect(&self.keys, rows)
            .map(ExecutedBatch::new::<K>)
    }
}
