use std::collections::HashMap;

use array_tool::vec::Shift;

use super::SparseSet;

#[derive(Default, Debug)]
pub struct Freelist {
    pub used: SparseSet,
    pub free: Vec<usize>,
    pub alias: HashMap<usize, usize>,
}

impl Freelist {
    pub fn len(&self) -> usize {
        self.used.len() + self.free.len()
    }

    pub fn reserve(&mut self) -> usize {
        let index = if self.free.is_empty() {
            self.used.len()
        } else {
            self.free.shift().unwrap()
        };

        self.used.add(index);
        index
    }

    pub fn free(&mut self, n: usize) {
        if self.used.is_member(n) {
            self.used.remove(n);
            self.free.push(n);
        }
    }

    pub fn is_valid(&self, n: usize) -> bool {
        if self.free.is_empty() {
            n < self.used.len()
        } else {
            !self.free.contains(&n)
        }
    }

    pub fn reserve_alias(&mut self, n: usize) -> Option<usize> {
        if self.alias.contains_key(&n) {
            return None;
        }

        let o = self.reserve();
        self.alias.insert(n, o);
        Some(n)
    }

    pub fn is_alias_valid(&self, n: usize) -> bool {
        self.alias.contains_key(&n)
    }

    pub fn free_alias(&mut self, n: usize) {
        if let Some(o) = self.alias.get(&n) {
            self.free(*o);
        }

        self.alias.remove(&n);
    }

    pub fn safe_free(&mut self, n: usize) {
        if self.is_alias_valid(n) {
            self.free_alias(n);
        } else if self.is_valid(n) {
            self.free(n)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Freelist;

    #[test]
    fn freelist_test() {
        let mut freelist = Freelist::default();
        assert_eq!(0, freelist.reserve());
        assert_eq!(1, freelist.reserve());
        assert_eq!(2, freelist.reserve());
        assert_eq!(3, freelist.reserve());
        assert!(freelist.is_valid(2));
        freelist.free(2);
        assert!(!freelist.is_valid(2));
        assert_eq!(2, freelist.reserve());
        assert!(freelist.is_valid(2));
        assert_eq!(4, freelist.reserve());
        freelist.free(1);
        freelist.free(4);
        assert_eq!(1, freelist.reserve());
        assert_eq!(4, freelist.reserve());
        assert_eq!(5, freelist.reserve());
        assert!(!freelist.is_valid(6));
    }
}
