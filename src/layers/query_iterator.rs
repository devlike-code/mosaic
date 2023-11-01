use array_tool::vec::Intersect;

use crate::internals::EntityId;

pub struct QueryIterator {
    elements: Vec<EntityId>,
}

impl Into<QueryIterator> for Vec<EntityId> {
    fn into(self) -> QueryIterator {
        QueryIterator { elements: self }
    }
}

impl<'a> IntoIterator for &'a QueryIterator {
    type Item = &'a EntityId;

    type IntoIter = std::slice::Iter<'a, EntityId>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl FromIterator<EntityId> for QueryIterator {
    fn from_iter<T: IntoIterator<Item = EntityId>>(iter: T) -> Self {
        QueryIterator { elements: iter.into_iter().collect() }
    }
}

impl QueryIterator {
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn sort(&mut self) {
        self.elements.sort();
    }

    pub fn as_slice(&self) -> &[EntityId] {
        self.elements.as_slice()
    }

    pub fn as_vec(&self) -> Vec<EntityId> {
        self.elements.clone()
    }

    pub fn union(mut self, other: QueryIterator) -> Self {
        self.elements.extend(other.as_slice());
        self
    }

    pub fn intersect(mut self, other: QueryIterator) -> Self {
        self.elements = self.elements.intersect(other.as_vec());
        self
    }
}