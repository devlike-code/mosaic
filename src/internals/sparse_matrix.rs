use std::collections::{HashMap, HashSet, VecDeque};

use super::datatypes::EntityId;

pub type SparseMatrixHashMap = HashMap<EntityId, HashMap<EntityId, Vec<EntityId>>>;
pub type SparseMatrixArrowIdentity = HashMap<EntityId, Vec<(EntityId, EntityId)>>;
pub type Path = Vec<EntityId>;

pub trait Matrix {
    fn is_empty(&self) -> bool;
    fn add_node(&mut self, node: EntityId);
    fn add_edge(&mut self, id: EntityId, src: EntityId, tgt: EntityId);
    fn remove_edge(&mut self, id: EntityId);
    fn remove_edges(&mut self, src: EntityId, tgt: EntityId);
    fn check_edge(&self, src: EntityId, tgt: EntityId) -> bool;
    fn get_all_nodes(&self) -> Vec<EntityId>;
    fn get_all_edges(&self) -> Vec<EntityId>;
}

#[derive(Default, Debug, Clone)]
pub struct UndirectedAdjacencyMatrix {
    pub adjacency: SparseMatrixHashMap, // a -> (b, e)
    pub edges: SparseMatrixArrowIdentity,     // e -> (a, b)
}

impl Matrix for UndirectedAdjacencyMatrix {
    fn is_empty(&self) -> bool {
        self.adjacency.is_empty()
    }

    fn add_node(&mut self, node: EntityId) {
        self.adjacency.insert(node, HashMap::default());
    }

    fn add_edge(&mut self, id: EntityId, src: EntityId, tgt: EntityId) {
        if !self.adjacency.contains_key(&src) {
            self.add_node(src);
        }

        if !self.adjacency.contains_key(&tgt) {
            self.add_node(tgt);
        }

        if !self.adjacency.get(&src).unwrap().contains_key(&tgt) {
            self.adjacency.get_mut(&src).unwrap().insert(tgt, vec![ id ]);
        } else {
            self.adjacency.get_mut(&src).unwrap().get_mut(&tgt).unwrap().push(id);
        }

        if !self.edges.contains_key(&id) {
            self.edges.insert(id, vec![ (src, tgt) ]);
        } else {
            self.edges.get_mut(&id).unwrap().push((src, tgt));
        }
        
        if !self.adjacency.get(&tgt).unwrap().contains_key(&src) {
            self.adjacency.get_mut(&tgt).unwrap().insert(src, vec![ id ]);
        } else {
            self.adjacency.get_mut(&tgt).unwrap().get_mut(&src).unwrap().push(id);
        }

        if !self.edges.contains_key(&id) {
            self.edges.insert(id, vec![ (tgt, src) ]);
        } else {
            self.edges.get_mut(&id).unwrap().push((tgt, src));
        }
    }

    fn remove_edge(&mut self, id: EntityId) {
        if let Some(kv) = self.edges.get(&id) {
            for (k, v) in kv {
                if let Some(key) = self.adjacency.get_mut(&k) {
                    key.remove(v);
                }   

                if let Some(key) = self.adjacency.get_mut(&v) {
                    key.remove(k);
                }   
            }
        }

        self.edges.remove(&id);
    }

    fn remove_edges(&mut self, src: EntityId, tgt: EntityId) {
        if let Some(kv) = self.adjacency.get_mut(&src) {
            if let Some(ids) = kv.get(&tgt) {
                for id in ids {
                    self.edges.remove(id);
                }                
            }

            kv.remove(&tgt);
        }

        if let Some(kv) = self.adjacency.get_mut(&tgt) {
            if let Some(ids) = kv.get(&src) {
                for id in ids {
                    self.edges.remove(id);
                }
            }

            kv.remove(&src);
        }

    }

    fn check_edge(&self, src: EntityId, tgt: EntityId) -> bool {
        if src == tgt {
            return true;
        }

        if let Some(adj_map) = self.adjacency.get(&src) {
            adj_map.contains_key(&tgt)
        } else {
            if let Some(adj_map) = self.adjacency.get(&tgt) {
                adj_map.contains_key(&src)
            } else {
                false
            }
        }
    }

    fn get_all_nodes(&self) -> Vec<EntityId> {
        self.adjacency.keys().cloned().collect()       
    }

    fn get_all_edges(&self) -> Vec<EntityId> {
        self.edges.keys().cloned().collect()
    }
    
}

impl UndirectedAdjacencyMatrix {
    pub fn are_adjacent(&self, src: EntityId, tgt: EntityId) -> bool {
        self.check_edge(src, tgt)
    }

    pub fn neighbor_count(&self, src: EntityId) -> usize {
        if let Some(adj_map) = self.adjacency.get(&src) {
            adj_map.len()
        } else {
            0
        }
    }

    pub fn neighbors(&self, src: EntityId) -> Vec<EntityId> {
        if let Some(adj_map) = self.adjacency.get(&src) {
            Vec::from_iter(adj_map.keys().cloned().into_iter())
        } else {
            vec![]
        }
    }

    pub fn edges(&self, src: EntityId) -> Vec<EntityId> {
        if let Some(adj_map) = self.adjacency.get(&src) {
            Vec::from_iter(adj_map.values().flatten().cloned().into_iter())
        } else {
            vec![]
        }
    }

    pub fn dfs(&self, src: EntityId) -> Vec<Path> {
        fn dfs_rec(
            this: &UndirectedAdjacencyMatrix,
            results: &mut Vec<Path>,
            freelist: &mut VecDeque<EntityId>,
            finished: &mut HashSet<EntityId>,
            history: &mut Vec<EntityId>,
        ) {
            while let Some(current_node) = freelist.pop_back() {
                finished.insert(current_node);
                history.push(current_node);

                let neighbors = this.neighbors(current_node);
                if neighbors.is_empty() {
                    results.push(history.clone());
                } else {
                    for neighbor in neighbors {
                        if !finished.contains(&neighbor) {
                            freelist.push_back(neighbor);
                            dfs_rec(this, results, freelist, finished, history);
                            freelist.pop_back();
                        } else {
                            //history.push(neighbor);
                            results.push(history.clone());
                            history.pop();
                        }
                    }
                }

                if let Some(popped) = history.pop() {
                    finished.remove(&popped);
                }
            }
        }

        let mut results: Vec<Path> = vec![];
        let mut freelist = VecDeque::default();
        let mut finished = HashSet::new();
        let mut history = vec![];
        freelist.push_back(src);

        dfs_rec(
            self,
            &mut results,
            &mut freelist,
            &mut finished,
            &mut history,
        );
        results
    }
}

#[derive(Default, Debug, Clone)]
pub struct AdjacencyMatrix {
    pub adjacency: SparseMatrixHashMap, // a -> (b, e)
    pub edges: SparseMatrixArrowIdentity,     // e -> (a, b)
}

impl Matrix for AdjacencyMatrix {
    fn is_empty(&self) -> bool {
        self.adjacency.is_empty()
    }

    fn add_node(&mut self, node: EntityId) {
        self.adjacency.insert(node, HashMap::default());
    }

    fn add_edge(&mut self, id: EntityId, src: EntityId, tgt: EntityId) {
        if !self.adjacency.contains_key(&src) {
            self.add_node(src);
        }

        if !self.adjacency.contains_key(&tgt) {
            self.add_node(tgt);
        }

        if !self.adjacency.get(&src).unwrap().contains_key(&tgt) {
            self.adjacency.get_mut(&src).unwrap().insert(tgt, vec![ id ]);
        } else {
            self.adjacency.get_mut(&src).unwrap().get_mut(&tgt).unwrap().push(id);
        }

        if !self.edges.contains_key(&id) {
            self.edges.insert(id, vec![ (src, tgt) ]);
        } else {
            self.edges.get_mut(&id).unwrap().push((src, tgt));
        }
    }

    fn remove_edge(&mut self, id: EntityId) {
        if let Some(adj_map) = self.edges.get(&id) {
            for (src, _) in adj_map {
                if let Some(tgts) = self.adjacency.get_mut(src) {
                    for (_, ids) in tgts {
                        if let Some(index) = ids.iter().position(|v| *v == id) {
                            ids.swap_remove(index);
                        }
                    }
                }
            }
        }
        
        self.edges.remove(&id);
    }

    fn remove_edges(&mut self, src: EntityId, tgt: EntityId) {
        if let Some(kv) = self.adjacency.get_mut(&src) {
            if let Some(ids) = kv.get(&tgt) {
                for id in ids {
                    self.edges.remove(id);
                }
            }

            kv.remove(&tgt);
        }
    }

    fn check_edge(&self, src: EntityId, tgt: EntityId) -> bool {
        if src == tgt {
            return true;
        }

        if let Some(adj_map) = self.adjacency.get(&src) {
            adj_map.contains_key(&tgt)
        } else {
            false
        }
    }

    fn get_all_nodes(&self) -> Vec<EntityId> {
        self.adjacency.keys().cloned().collect()       
    }

    fn get_all_edges(&self) -> Vec<EntityId> {
        self.edges.keys().cloned().collect()
    }    
    
}

impl AdjacencyMatrix {
    pub fn are_adjacent(&self, src: EntityId, tgt: EntityId) -> bool {
        self.check_edge(src, tgt)
    }

    fn neighbor_count(&self, src: EntityId) -> usize {
        if let Some(adj_set) = self.adjacency.get(&src) {
            adj_set.len()
        } else {
            0
        }
    }

    fn neighbors(&self, src: EntityId) -> Vec<EntityId> {
        if let Some(adj_map) = self.adjacency.get(&src) {
            Vec::from_iter(adj_map.keys().filter(|e| !adj_map.get(*e).unwrap().is_empty()).cloned().into_iter())
        } else {
            vec![]
        }
    }

    fn edges(&self, src: EntityId) -> Vec<EntityId> {
        if let Some(adj_map) = self.adjacency.get(&src) {
            Vec::from_iter(adj_map.values().flatten().cloned().into_iter())
        } else {
            vec![]
        }
    }

    pub fn dfs(&self, src: EntityId) -> Vec<Path> {
        fn dfs_rec(
            this: &AdjacencyMatrix,
            results: &mut Vec<Path>,
            freelist: &mut VecDeque<EntityId>,
            finished: &mut HashSet<EntityId>,
            history: &mut Vec<EntityId>,
        ) {
            while let Some(current_node) = freelist.pop_back() {
                finished.insert(current_node);
                history.push(current_node);

                let neighbors = this.neighbors(current_node);
                if neighbors.is_empty() {
                    results.push(history.clone());
                } else {
                    for neighbor in neighbors {
                        if !finished.contains(&neighbor) {
                            freelist.push_back(neighbor);
                            dfs_rec(this, results, freelist, finished, history);
                            freelist.pop_back();
                        } else {
                            history.push(neighbor);
                            results.push(history.clone());
                            history.pop();
                        }
                    }
                }

                if let Some(popped) = history.pop() {
                    finished.remove(&popped);
                }
            }
        }

        let mut results: Vec<Path> = vec![];
        let mut freelist = VecDeque::default();
        let mut finished = HashSet::new();
        let mut history = vec![];
        freelist.push_back(src);

        dfs_rec(
            self,
            &mut results,
            &mut freelist,
            &mut finished,
            &mut history,
        );
        results
    }
}

#[derive(Default, Debug, Clone)]
pub struct BidirectionalMatrix {
    pub forward: AdjacencyMatrix,
    pub backward: AdjacencyMatrix,
}

impl Matrix for BidirectionalMatrix {
    fn is_empty(&self) -> bool {
        self.forward.is_empty()
    }

    fn add_node(&mut self, node: EntityId) {
        self.forward.add_node(node);
        self.backward.add_node(node);
    }

    fn add_edge(&mut self, id: EntityId, src: EntityId, tgt: EntityId) {
        self.forward.add_edge(id, src, tgt);
        self.backward.add_edge(id, tgt, src);
    }

    fn remove_edge(&mut self, id: EntityId) {
        self.forward.remove_edge(id);
        self.backward.remove_edge(id);
    }

    fn remove_edges(&mut self, src: EntityId, tgt: EntityId) {
        self.forward.remove_edges(src, tgt);
        self.backward.remove_edges(src, tgt);
    }

    fn check_edge(&self, src: EntityId, tgt: EntityId) -> bool {
        self.forward.check_edge(src, tgt)
    }

    fn get_all_nodes(&self) -> Vec<EntityId> {
        self.forward.get_all_nodes()
    }

    fn get_all_edges(&self) -> Vec<EntityId> {
        self.forward.get_all_edges()
    }
    
}

impl BidirectionalMatrix {
    
    pub fn out_degree(&self, src: EntityId) -> usize {
        self.forward.neighbor_count(src)
    }

    pub fn in_degree(&self, src: EntityId) -> usize {
        self.backward.neighbor_count(src)
    }

    pub fn are_adjacent(&self, src: EntityId, tgt: EntityId) -> bool {
        self.check_edge(src, tgt)
    }

    pub fn get_front_neighbors(&self, src: EntityId) -> Vec<EntityId> {
        self.forward.neighbors(src)
    }

    pub fn get_back_neighbors(&self, src: EntityId) -> Vec<EntityId> {
        self.backward.neighbors(src)
    }

    pub fn reach_forward(&self, src: EntityId) -> Vec<Path> {
        self.forward.dfs(src)
    }

    pub fn reach_backward(&self, src: EntityId) -> Vec<Path> {
        self.backward.dfs(src)
    }

    pub fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> bool {
        let reach = self.reach_forward(src);
        reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .collect::<Vec<_>>()
            .len()
            > 0
    }

    pub fn reach_backward_until(&self, src: EntityId, tgt: EntityId) -> bool {
        let reach = self.reach_backward(src);
        reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .collect::<Vec<_>>()
            .len()
            > 0
    }

    pub fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool {
        self.reach_forward_until(src, tgt)
    }

    pub fn edges_from(&self, src: EntityId) -> Vec<EntityId> {
        self.forward.edges(src)
    }

    pub fn edges_into(&self, src: EntityId) -> Vec<EntityId> {
        self.backward.edges(src)
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod sparse_matrix_testing {
    use super::AdjacencyMatrix;
    use super::BidirectionalMatrix;
    use super::Matrix;

    #[test]
    fn test_adding_an_edge_builds_adjacency() {
        let mut mat = AdjacencyMatrix::default();
        mat.add_edge(3, 1, 2);
        assert!(mat.are_adjacent(1, 2));
        assert!(!mat.are_adjacent(2, 3));
        mat.add_edge(4, 2, 3);
        assert!(mat.are_adjacent(1, 2));
        assert!(mat.are_adjacent(2, 3));
        assert!(!mat.are_adjacent(3, 1));
        mat.add_edge(5, 3, 1);
        assert!(mat.are_adjacent(1, 2));
        assert!(mat.are_adjacent(2, 3));
        assert!(mat.are_adjacent(3, 1));
    }

    #[test]
    fn test_building_bidir_matrices_works() {
        let mut mat = BidirectionalMatrix::default();
        mat.add_edge(3, 1, 2);
        assert!(mat.are_adjacent(1, 2));
        assert!(!mat.are_adjacent(2, 3));
        mat.add_edge(4, 2, 3);
        assert!(mat.are_adjacent(1, 2));
        assert!(mat.are_adjacent(2, 3));
        assert!(!mat.are_adjacent(3, 1));
        mat.add_edge(5, 3, 1);
        assert!(mat.are_adjacent(1, 2));
        assert!(mat.are_adjacent(2, 3));
        assert!(mat.are_adjacent(3, 1));
    }

    #[test]
    fn test_dfs_on_trees() {
        let mut mat = BidirectionalMatrix::default();
        /*

            1 ----> 2 ----> 5 ----> 6
                    |
                    |
                    v
                    3 ----> 4

        */

        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 3);
        mat.add_edge(12, 3, 4);
        mat.add_edge(13, 2, 5);
        mat.add_edge(14, 5, 6);

        let paths = mat.reach_forward(1);
        println!("{:?}", paths);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_dfs_with_loops() {
        let mut mat = BidirectionalMatrix::default();
        /*

            1 ----> 2
             ^      |
              \     |
               \    v
                --- 3 ----> 4

        */

        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 3);
        mat.add_edge(12, 3, 1);
        mat.add_edge(13, 3, 4);
        mat.add_edge(14, 4, 1);

        let paths = mat.reach_forward(1);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_dfs_with_repeated_adds() {
        let mut mat = BidirectionalMatrix::default();
        /*

            1 ----> 2 ----- x
                    | \     |
                    |   \   |
                    v     \ v
                    x ----> 4 ----> 5

        */

        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 4);
        mat.add_edge(12, 2, 4);
        mat.add_edge(13, 4, 5);

        let paths = mat.reach_forward(1);
        assert_eq!(paths.len(), 1);
    }

    #[test]
    fn test_dfs_complex() {
        let mut mat = BidirectionalMatrix::default();
        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 1, 4);
        mat.add_edge(12, 1, 9);
        mat.add_edge(13, 2, 3);
        mat.add_edge(14, 2, 6);
        mat.add_edge(15, 3, 5);
        mat.add_edge(16, 3, 7);
        mat.add_edge(17, 3, 8);
        mat.add_edge(18, 4, 6);
        mat.add_edge(19, 5, 2);
        mat.add_edge(20, 5, 6);
        mat.add_edge(21, 6, 1);
        mat.add_edge(22, 7, 5);
        mat.add_edge(23, 7, 8);
        mat.add_edge(24, 8, 1);
        mat.add_node(9);
        let paths = mat.reach_forward(1);
        assert_eq!(paths.len(), 9);
    }

    #[test]
    fn test_dfs_with_converging_ends() {
        let mut mat = BidirectionalMatrix::default();
        /*

            1 ----> 2 ----- 6
                    |       |
                    |       |
                    v       v
                    3 ----> 4 ----> 5

        */

        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 3);
        mat.add_edge(12, 2, 6);
        mat.add_edge(13, 6, 4);
        mat.add_edge(14, 3, 4);
        mat.add_edge(15, 4, 5);

        let paths = mat.reach_forward(1);
        assert_eq!(paths.len(), 2);
    }

    #[test]
    fn test_simple_reachability() {
        let mut mat = BidirectionalMatrix::default();
        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 3);
        mat.add_edge(12, 3, 4);
        assert!(mat.are_reachable(1, 4));
        mat.remove_edge(12);
        assert!(!mat.are_reachable(1, 4));
    }

    #[test]
    fn test_deleting_self_loops() {
        let mut mat = BidirectionalMatrix::default();
        mat.add_edge(10, 1, 1);
        mat.add_edge(11, 1, 1);
        mat.add_edge(12, 1, 1);
        println!("{:?}", mat.edges_from(1));
        mat.remove_edge(11);
        println!("{:?}", mat.edges_from(1));

        assert!(mat.check_edge(1, 1));
    }

    #[test]
    fn test_add_node_in_closed_adjacency_matrix() {
        let mut mat = BidirectionalMatrix::default();
        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 3);
        mat.add_edge(12, 4, 5);
        mat.add_edge(13, 5, 6);
        mat.add_edge(14, 3, 4);
        assert!(!mat.are_adjacent(1, 6));
        assert!(mat.are_reachable(1, 6));
    }

    #[test]
    fn test_all_adjacent_edges_are_also_reachable() {
        let mut mat = BidirectionalMatrix::default();
        mat.add_edge(10, 1, 2);
        mat.add_edge(11, 2, 3);
        mat.add_edge(12, 3, 1);

        assert!(mat.are_reachable(1, 2));
        assert!(mat.are_reachable(2, 3));
        assert!(mat.are_reachable(3, 1));
        assert!(mat.are_reachable(1, 3));
        assert!(mat.are_reachable(2, 1));
    }
}
