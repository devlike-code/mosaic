use std::{
    collections::{HashSet, VecDeque},
    thread::sleep,
};

use array_tool::vec::Uniq;
use itertools::Itertools;

use crate::{
    internals::{engine_state, Block, BrickEditing, EngineState, EntityId, Tile},
    layers::{indirection::Indirection, querying::Querying, tiling::Tiling},
};

use super::{accessing::Accessing, query_iterator::QueryIterator};
pub type Path = Vec<EntityId>;

pub trait Traversing {
    /// traversing graph using Depth First Search
    fn dfs(&self, src: EntityId) -> Vec<Path>;   
    fn reach_forward(&self, src: EntityId) -> Vec<Path>;
    fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> bool;
    fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool;
    
}

impl Traversing for EngineState {
    fn reach_forward(&self, src: EntityId) -> Vec<Path> {
        self.dfs(src)
    }

    fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> bool {
        let reach = self.reach_forward(src);
        println!("DFS reach forward: {:?}", reach);
        reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .collect::<Vec<_>>()
            .len()
            > 0
    }

    fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool {
        self.reach_forward_until(src, tgt)
    }

    fn dfs(&self, src: EntityId) -> Vec<Path> {
        fn dfs_rec(
            engine_state: &EngineState,
            src: EntityId,
            results: &mut Vec<Path>,
            freelist: &mut VecDeque<EntityId>,
            finished: &mut HashSet<EntityId>,
            history: &mut Vec<EntityId>,
        ) {
            while let Some(current_node) = freelist.pop_back() {
                println!("current_node is: {:?}", current_node);

                finished.insert(current_node);
                history.push(current_node);

                let neighbors = engine_state
                    .query_neighbors(current_node)
                    .into_iter()
                    .cloned()
                    .collect_vec();
                println!("Neighbors: {:?}", neighbors);

                if neighbors.is_empty() {
                    results.push(history.clone());
                } else {
                    for neighbor in neighbors {
                        if !finished.contains(&neighbor) {
                            freelist.push_back(neighbor);
                            dfs_rec(
                                engine_state,
                                current_node,
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
            self,
            src,
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
