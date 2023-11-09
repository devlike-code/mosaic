use std::sync::Arc;

use array_tool::vec::{Intersect, Uniq};

use crate::internals::EntityId;

use super::{mosaic_engine::MosaicEngine, EngineState};

#[derive(Clone, Default)]
/// A query iterator is a thin wrapper around a vector of entity identifiers
pub struct QueryIterator {
    pub(crate) engine: Arc<EngineState>,
    pub(crate) elements: Vec<EntityId>,
}

impl std::fmt::Debug for QueryIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("QueryIterator")
            .field("elements", &self.elements)
            .finish()
    }
}

impl Into<QueryIterator> for (&Arc<EngineState>, Vec<EntityId>) {
    fn into(self) -> QueryIterator {
        QueryIterator {
            engine: Arc::clone(self.0),
            elements: self.1,
        }
    }
}

impl Into<QueryIterator> for (Arc<EngineState>, Vec<EntityId>) {
    fn into(self) -> QueryIterator {
        QueryIterator {
            engine: self.0,
            elements: self.1,
        }
    }
}

impl<'a> IntoIterator for &'a QueryIterator {
    type Item = &'a EntityId;

    type IntoIter = std::slice::Iter<'a, EntityId>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl QueryIterator {
    /// Wraps around the length of the current iterator
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Wraps around the `sort` function of the underlying vector
    pub fn sort(&mut self) {
        self.elements.sort();
    }

    /// Returns a slice of the underlying vector
    pub fn as_slice(&self) -> &[EntityId] {
        self.elements.as_slice()
    }

    /// Returns a clone of the underlying vector
    pub fn as_vec(&self) -> Vec<EntityId> {
        self.elements.clone()
    }

    /// Builds a union of this and another iterator
    pub fn union(mut self, other: QueryIterator) -> Self {
        self.elements.extend(other.as_slice());
        self.elements = self.elements.unique();
        self
    }

    /// Builds an intersection of this and another iterator
    pub fn intersect(mut self, other: QueryIterator) -> Self {
        self.elements = self.elements.intersect(other.as_vec());
        self
    }

    pub fn contains(&self, id: &EntityId) -> bool {
        self.elements.contains(id)
    }
}
