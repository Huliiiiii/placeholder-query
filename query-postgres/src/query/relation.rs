use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use placeholder_query_core::types::{Ident, ParamId};

use super::{
    Expr,
    columns::{Columns, FieldTransformer, Rebind},
    expr::ExprNode,
    mode::{self, Mode},
    params::{ArrayElement, Param, SqlType},
    select::ast::SelectAst,
};

#[doc(hidden)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnnestColumn {
    pub(crate) name: &'static str,
    pub(crate) param_id: ParamId,
    pub(crate) sql_type: SqlType,
}

impl UnnestColumn {
    #[doc(hidden)]
    pub fn new<T: ArrayElement>(name: &'static str, param: Param<Vec<T>>) -> Self {
        Self {
            name,
            param_id: param.id,
            sql_type: SqlType::Array(Box::new(T::ty())),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum RelationNode {
    Table { table: Ident, fields: Arc<[Ident]> },
    Unnest(Arc<[UnnestColumn]>),
    DerivedTable(Arc<SelectAst>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct RelationId(u64);

impl RelationId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);

        Self(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub(crate) id: RelationId,
    pub(crate) node: RelationNode,
}

impl Relation {
    pub(crate) fn table(table: Ident, fields: Vec<Ident>) -> Self {
        Self {
            id: RelationId::new(),
            node: RelationNode::Table {
                table,
                fields: fields.into(),
            },
        }
    }

    pub(crate) fn unnest(columns: Vec<UnnestColumn>) -> Self {
        Self {
            id: RelationId::new(),
            node: RelationNode::Unnest(columns.into()),
        }
    }

    pub(crate) fn derived_table(query: Arc<SelectAst>) -> Self {
        Self {
            id: RelationId::new(),
            node: RelationNode::DerivedTable(query),
        }
    }

    /// References one of this relation's output columns by position.
    #[doc(hidden)]
    pub fn column<T>(&self, field: u16) -> Expr<T> {
        Expr::from_node(ExprNode::Column {
            relation: self.id,
            field,
        })
    }

    pub(crate) fn bind<M: Mode, C: Columns<M>>(
        &self,
        columns: &C,
    ) -> <C as Rebind>::Rebound<mode::Expr> {
        columns.map_fields(&mut BindColumns {
            relation: self,
            index: 0,
        })
    }

    pub(crate) fn freshen(&self, bindings: &[(RelationId, RelationId)]) -> Self {
        let node = match &self.node {
            RelationNode::DerivedTable(query) => {
                RelationNode::DerivedTable(Arc::new(query.freshen(bindings).0))
            }
            relation_node => relation_node.clone(),
        };

        Self {
            id: RelationId::new(),
            node,
        }
    }
}

struct BindColumns<'a> {
    relation: &'a Relation,
    index: usize,
}

impl<M: Mode> FieldTransformer<M, mode::Expr> for BindColumns<'_> {
    fn transform<T>(&mut self, _: &M::Field<T>) -> Expr<T> {
        let column = self
            .relation
            .column(u16::try_from(self.index).expect("relation exceeds 65536 output columns"));

        self.index += 1;

        column
    }
}

impl PartialEq for Relation {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Relation {}
