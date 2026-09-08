use crate::query::relation::{Relation, RelationId};

use super::ast::{Join, SelectBody};

#[derive(Clone, Copy)]
pub(super) struct Binding<'a> {
    pub relation: &'a Relation,
    pub alias: usize,
}

pub(super) fn bind_sources<'a>(body: &'a SelectBody, next_alias: &mut usize) -> Vec<Binding<'a>> {
    std::iter::once(&body.from)
        .chain(body.joins.iter().map(|join| match join {
            Join::Inner { relation, .. } | Join::Lateral(relation) => relation,
        }))
        .map(|relation| {
            let alias = *next_alias;
            *next_alias += 1;
            Binding { relation, alias }
        })
        .collect()
}

pub(super) struct Scope<'a> {
    pub parent: Option<&'a Scope<'a>>,
    pub bindings: &'a [Binding<'a>],
}

impl Scope<'_> {
    pub fn prefix(&self, count: usize) -> Self {
        Self {
            parent: self.parent,
            bindings: &self.bindings[..count],
        }
    }

    pub fn resolve(&self, relation_id: RelationId) -> Binding<'_> {
        self.bindings
            .iter()
            .rev()
            .find(|binding| binding.relation.id == relation_id)
            .copied()
            .unwrap_or_else(|| {
                self.parent
                    .expect("column refers to a source outside this query")
                    .resolve(relation_id)
            })
    }
}
