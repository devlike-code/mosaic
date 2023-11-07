use itertools::Itertools;
use std::collections::{HashSet, VecDeque};

use crate::{
    internals::{EngineState, EntityId},
    layers::querying::Querying,
};

pub type Path = Vec<EntityId>;

#[derive(Debug, Default, PartialEq, Clone)]
pub enum Traversal {
    #[default]
    Forward,
    Backward,
    Both,
}

pub trait Traversing {
    fn out_degree(&self, src: EntityId) -> usize;
    fn in_degree(&self, src: EntityId) -> usize;
    /// traversing graph using Depth First Search
    fn dfs(&self, src: EntityId, traversal: Traversal) -> Vec<Path>;
    fn reach_forward(&self, src: EntityId) -> Vec<Path>;
    fn reach_backward(&self, src: EntityId) -> Vec<Path>;
    fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> Option<Path>;
    fn reach_backward_until(&self, src: EntityId, tgt: EntityId) -> Option<Path>;
    fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool;
}

impl Traversing for EngineState {
    fn out_degree(&self, src: EntityId) -> usize {
        self.query_forward_neighbors(src).len()
    }

    fn in_degree(&self, src: EntityId) -> usize {
        self.query_backward_neighbors(src).len()
    }

    fn reach_forward(&self, src: EntityId) -> Vec<Path> {
        self.dfs(src, Traversal::Forward)
    }

    fn reach_backward(&self, src: EntityId) -> Vec<Path> {
        self.dfs(src, Traversal::Backward)
    }

    fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> Option<Path> {
        let reach = self.reach_forward(src);
        //println!("DFS reach forward: {:?}", reach);
        let path = reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .cloned()
            .collect_vec();
        if path.len() > 0 {
            Some(path)
        } else {
            None
        }
    }

    fn reach_backward_until(&self, src: EntityId, tgt: EntityId) -> Option<Path> {
        let reach = self.reach_backward(src);
        //println!("DFS reach forward: {:?}", reach);
        let path = reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .cloned()
            .collect_vec();
        if path.len() > 0 {
            Some(path)
        } else {
            None
        }
    }

    fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool {
        self.reach_forward_until(src, tgt).is_some()
    }

    fn dfs(&self, src: EntityId, traversal: Traversal) -> Vec<Path> {
        fn dfs_rec(
            traversal: &Traversal,
            engine_state: &EngineState,
            results: &mut Vec<Path>,
            freelist: &mut VecDeque<EntityId>,
            finished: &mut HashSet<EntityId>,
            history: &mut Vec<EntityId>,
        ) {
            // println!("results: {:?}", results);
            // println!("freelist: {:?}", freelist);
            // println!("finished: {:?}", finished);
            // println!("history: {:?}", history);

            while let Some(current_node) = freelist.pop_back() {
                // println!("current_node is: {:?}", current_node);
                finished.insert(current_node);
                history.push(current_node);

                let neighbors = match traversal {
                    Traversal::Forward => engine_state.query_forward_neighbors(current_node),
                    Traversal::Backward => engine_state.query_backward_neighbors(current_node),
                    Traversal::Both => engine_state.query_neighbors(current_node),
                    //println!("Neighbors: {:?}", neighbors);
                }
                .into_iter()
                .cloned()
                .collect_vec();
                if neighbors.is_empty() {
                    results.push(history.clone());
                } else {
                    for neighbor in neighbors {
                        if !finished.contains(&neighbor) {
                            freelist.push_back(neighbor);
                            dfs_rec(
                                traversal,
                                engine_state,
                                results,
                                freelist,
                                finished,
                                history,
                            );
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
            &traversal,
            self,
            &mut results,
            &mut freelist,
            &mut finished,
            &mut history,
        );
        results
    }
}
#[cfg(test)]
mod traversing_tests {
    use crate::{internals::engine_state::EngineState, layers::traversing::Traversing};

    #[test]
    fn test_simple_reachability() {
        let engine_state = EngineState::default();

        let _ = engine_state.add_component_types("Object: void; Arrow: void;");

        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let d = engine_state.create_object_raw("Object".into(), vec![]);
        let e = engine_state.create_object_raw("Object".into(), vec![]);
        /*
            a -- x ---> b ----- y
                        |       |
                        |       |
                        v ----> d -- z --> e

        */
        let x = engine_state
            .create_arrow(a, b, "Arrow".into(), vec![])
            .unwrap();
        let y = engine_state
            .create_arrow(b, d, "Arrow".into(), vec![])
            .unwrap();
        let v = engine_state
            .create_arrow(b, d, "Arrow".into(), vec![])
            .unwrap();
        let z = engine_state
            .create_arrow(d, e, "Arrow".into(), vec![])
            .unwrap();
        println!("{a} -- {x} ---> {b} ----- {y}");
        println!("            |       |");
        println!("            |       |");
        println!("            {v} ----> {d} -- {z} --> {e}");

        assert!(engine_state.are_reachable(a, e));
        engine_state.destroy_arrow(v);
        assert!(engine_state.are_reachable(a, e));
        engine_state.destroy_arrow(y);
        assert!(!engine_state.are_reachable(a, e));
    }
}
