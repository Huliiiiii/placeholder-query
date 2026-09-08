use std::{
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    sync::Arc,
};

use lru::LruCache;
use parking_lot::Mutex;
use placeholder_query_postgres::{Statement, query::params::SqlType};
use tokio::sync::OnceCell;
use tokio_postgres::{Client, Error, Statement as PreparedStatement};

/// Identifies the query template by its ptr.
struct CacheKey(Arc<Statement>);

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for CacheKey {}

pub(crate) struct StatementCache {
    entries: Mutex<LruCache<CacheKey, Arc<OnceCell<PreparedStatement>>>>,
}

impl StatementCache {
    pub(crate) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            entries: Mutex::new(LruCache::new(capacity)),
        }
    }

    pub(crate) fn resize(&mut self, capacity: NonZeroUsize) {
        self.entries.get_mut().resize(capacity);
    }

    pub(crate) fn clear(&self) {
        self.entries.lock().clear();
    }

    pub(crate) async fn prepare(
        &self,
        client: &Client,
        statement: &Arc<Statement>,
    ) -> Result<PreparedStatement, Error> {
        let entry = {
            let mut entries = self.entries.lock();
            entries
                .get_or_insert(CacheKey(statement.clone()), || Arc::new(OnceCell::new()))
                .clone()
        };

        entry
            .get_or_try_init(|| async {
                let types = statement
                    .params()
                    .iter()
                    .map(|param| to_postgres_type(&param.ty))
                    .collect::<Vec<_>>();
                client.prepare_typed(statement.sql(), &types).await
            })
            .await
            .cloned()
    }
}

fn to_postgres_type(ty: &SqlType) -> tokio_postgres::types::Type {
    use tokio_postgres::types::Type;

    match ty {
        SqlType::Bool => Type::BOOL,
        SqlType::Int2 => Type::INT2,
        SqlType::Int4 => Type::INT4,
        SqlType::Int8 => Type::INT8,
        SqlType::Float4 => Type::FLOAT4,
        SqlType::Float8 => Type::FLOAT8,
        SqlType::Text => Type::TEXT,
        SqlType::Bytea => Type::BYTEA,
        SqlType::Array(element) => match element.as_ref() {
            SqlType::Bool => Type::BOOL_ARRAY,
            SqlType::Int2 => Type::INT2_ARRAY,
            SqlType::Int4 => Type::INT4_ARRAY,
            SqlType::Int8 => Type::INT8_ARRAY,
            SqlType::Float4 => Type::FLOAT4_ARRAY,
            SqlType::Float8 => Type::FLOAT8_ARRAY,
            SqlType::Text => Type::TEXT_ARRAY,
            SqlType::Bytea => Type::BYTEA_ARRAY,
            SqlType::Array(_) => panic!("nested PostgreSQL arrays are unsupported"),
        },
    }
}
