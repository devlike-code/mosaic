use super::datatypes::EntityId;
use std::collections::HashMap;

#[derive(Debug, Clone, Default, PartialEq)]
/// An implementation of a sparse set.
pub struct SparseSet {
    order_max: usize,
    order_array: Vec<EntityId>,
    index_array: HashMap<EntityId, EntityId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// A helper for working with sparse set operations.
pub(crate) struct DenseSparsePair {
    dense_index: EntityId,
    sparse_index: EntityId,
}

impl SparseSet {
    pub fn new() -> Self {
        SparseSet {
            order_max: 0,
            order_array: Vec::default(),
            index_array: HashMap::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.order_max as usize
    }

    pub(crate) fn get_pair_by_pos(&self, pos: EntityId) -> Option<DenseSparsePair> {
        let dn = self.order_array[pos as usize];
        self.get_pair_by_index(dn)
    }

    pub(crate) fn get_pair_by_index(&self, sparse: EntityId) -> Option<DenseSparsePair> {
        if self.is_member(sparse) {
            let index = self.index_array[&sparse];
            assert!(index > 0);
            let dense = index - 1;
            Some(DenseSparsePair {
                dense_index: dense,
                sparse_index: sparse,
            })
        } else {
            None
        }
    }

    pub fn add(&mut self, i: EntityId) {
        self.order_array.insert(self.order_max as usize, i);

        self.order_max += 1;
        self.index_array.insert(i, self.order_max as usize);
    }

    pub(crate) fn swap(&mut self, a: &DenseSparsePair, b: &DenseSparsePair) {
        self.order_array[a.dense_index as usize] = b.sparse_index;
        self.order_array[b.dense_index as usize] = a.sparse_index;
        self.index_array.insert(a.sparse_index, b.dense_index + 1);
        self.index_array.insert(b.sparse_index, a.dense_index + 1);
    }

    pub fn remove(&mut self, i: EntityId) {
        if self.is_member(i) {
            let i_pair = self.get_pair_by_index(i);
            let n_pair = self.get_pair_by_pos(self.order_max - 1);

            if let (Some(i_p), Some(n_p)) = (i_pair, n_pair) {
                assert!(self.order_max > 0);
                self.order_max -= 1;
                self.swap(&i_p, &n_p);
                assert!(!self.order_array.is_empty());
                self.order_array.pop();
                self.index_array.insert(i_p.sparse_index, 0);
            }
        }
    }

    pub fn get_index(&self, i: EntityId) -> Option<EntityId> {
        if self.is_member(i) {
            Some(
                self.index_array
                    .get(&i)
                    .expect("Index not found (sparse_map:75)")
                    - 1,
            )
        } else {
            None
        }
    }

    pub fn is_member(&self, i: EntityId) -> bool {
        if self.order_max == 0 {
            return false;
        }

        if let Some(sp) = self.index_array.get(&i) {
            if *sp == 0 {
                return false;
            } else {
                let dn = *self
                    .order_array
                    .get((sp - 1) as usize)
                    .expect(format!("Dense map doesn't contain index {}", sp - 1).as_str());
                *sp <= self.order_max && dn == i
            }
        } else {
            return false;
        }
    }

    pub fn elements(&self) -> &Vec<EntityId> {
        &self.order_array
    }

    pub fn clear(&mut self) {
        self.order_max = 0;
    }
}

impl<'a> IntoIterator for &'a SparseSet {
    type Item = &'a EntityId;
    type IntoIter = std::slice::Iter<'a, EntityId>;

    fn into_iter(self) -> Self::IntoIter {
        self.order_array[..self.order_max as usize].iter()
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod sparse_set_testing {
    use crate::internals::sparse_set::SparseSet;

    #[test]
    fn test_sparse_set_addition() {
        let mut s = SparseSet::new();

        s.add(7);
        assert_eq!(s.order_array, [7]);
        assert_eq!(*s.index_array.get(&7).unwrap(), 1);

        s.add(5);
        assert_eq!(s.order_array, [7, 5]);
        assert_eq!(*s.index_array.get(&7).unwrap(), 1);
        assert_eq!(*s.index_array.get(&5).unwrap(), 2);

        s.add(9);
        assert_eq!(s.order_array, [7, 5, 9]);
        assert_eq!(*s.index_array.get(&7).unwrap(), 1);
        assert_eq!(*s.index_array.get(&5).unwrap(), 2);
        assert_eq!(*s.index_array.get(&9).unwrap(), 3);
    }

    #[test]
    fn test_sparse_set_is_member_works_around_one_offset() {
        let mut s = SparseSet::new();
        s.add(7);
        assert!(s.is_member(7));

        s.add(0);
        assert!(s.is_member(0));
    }

    #[test]
    fn test_sparse_set_swap() {
        let mut s = SparseSet::new();
        s.add(7);
        s.add(8);
        s.add(6);
        s.add(9);
        assert_eq!(s.order_max, 4);
        assert_eq!(s.order_array, [7, 8, 6, 9]);
        assert_eq!(*s.index_array.get(&6).unwrap(), 3);
        assert_eq!(*s.index_array.get(&7).unwrap(), 1);
        assert_eq!(*s.index_array.get(&8).unwrap(), 2);
        assert_eq!(*s.index_array.get(&9).unwrap(), 4);

        let p2 = s.get_pair_by_pos(2);
        let p3 = s.get_pair_by_index(3);

        if let (Some(p2), Some(p3)) = (p2, p3) {
            s.swap(&p2, &p3);
            assert_eq!(s.order_array, [7, 8, 9, 6]);
            assert_eq!(*s.index_array.get(&6).unwrap(), 4);
            assert_eq!(*s.index_array.get(&7).unwrap(), 1);
            assert_eq!(*s.index_array.get(&8).unwrap(), 2);
            assert_eq!(*s.index_array.get(&9).unwrap(), 3);
        }
    }

    #[test]
    fn test_sparse_set_removal() {
        let mut s = SparseSet::new();
        s.add(7);
        s.add(8);
        s.add(6);
        s.add(9);
        assert_eq!(s.order_max, 4);
        assert_eq!(s.order_array, [7, 8, 6, 9]);
        assert_eq!(*s.index_array.get(&6).unwrap(), 3);
        assert_eq!(*s.index_array.get(&7).unwrap(), 1);
        assert_eq!(*s.index_array.get(&8).unwrap(), 2);
        assert_eq!(*s.index_array.get(&9).unwrap(), 4);

        s.remove(8);
        assert_eq!(s.order_max, 3);
        assert_eq!(s.order_array, [7, 9, 6]);
        assert_eq!(*s.index_array.get(&6).unwrap(), 3);
        assert_eq!(*s.index_array.get(&7).unwrap(), 1);
        assert_eq!(*s.index_array.get(&8).unwrap(), 0);
        assert_eq!(*s.index_array.get(&9).unwrap(), 2);
    }
}
