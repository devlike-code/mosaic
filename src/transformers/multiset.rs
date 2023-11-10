use std::collections::{HashMap, HashSet};
use std::hash::Hash;

#[derive(Default, Debug, Clone)]
pub struct MultiSet<K, V> {
    pub set: HashMap<K, HashSet<V>>,
}

impl<K, V> MultiSet<K, V> 
where 
    K: Eq + Hash + Copy,
    V: Eq + PartialEq + Hash {

    pub fn insert(&mut self, k: K, v: V) {
        if !self.set.contains_key(&k) {
            self.set.insert(k, HashSet::new());
        }
        
        self.set.get_mut(&k).unwrap().insert(v);
    }

    pub fn contains_key(&self, k: K) -> bool {
        self.set.contains_key(&k)
    }

    pub fn contains_pair(&self, k: K, v: V) -> bool {
        self.set.get(&k).map(|s| s.contains(&v)).unwrap_or(false) 
    }
}