use std::sync::Arc;

use array_tool::vec::{Intersect, Uniq};
use itertools::Itertools;

use crate::internals::EntityId;

use super::EngineState;

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

impl From<(&Arc<EngineState>, Vec<EntityId>)> for QueryIterator {
    fn from(val: (&Arc<EngineState>, Vec<EntityId>)) -> Self {
        QueryIterator {
            engine: Arc::clone(val.0),
            elements: val.1,
        }
    }
}

impl From<(&Arc<EngineState>, Vec<&EntityId>)> for QueryIterator {
    fn from(val: (&Arc<EngineState>, Vec<&EntityId>)) -> Self {
        QueryIterator {
            engine: Arc::clone(val.0),
            elements: val.1.into_iter().cloned().collect_vec(),
        }
    }
}

impl From<(Arc<EngineState>, Vec<EntityId>)> for QueryIterator {
    fn from(val: (Arc<EngineState>, Vec<EntityId>)) -> Self {
        QueryIterator {
            engine: val.0,
            elements: val.1,
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

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
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
