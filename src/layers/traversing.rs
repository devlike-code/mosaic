use itertools::Itertools;
use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use crate::{
    internals::{query_iterator::QueryIterator, EngineState, EntityId},
    layers::querying::Querying,
};

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

    fn depth_first_search(&self, src: EntityId, traversal: Traversal) -> Vec<QueryIterator>;
    fn reach_forward(&self, src: EntityId) -> Vec<QueryIterator>;
    fn reach_backward(&self, src: EntityId) -> Vec<QueryIterator>;
    fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> Option<QueryIterator>;
    fn reach_backward_until(&self, src: EntityId, tgt: EntityId) -> Option<QueryIterator>;
    fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool;
}

impl Traversing for Arc<EngineState> {
    fn out_degree(&self, src: EntityId) -> usize {
        self.get_forward_neighbors(src).len()
    }

    fn in_degree(&self, src: EntityId) -> usize {
        self.get_backward_neighbors(src).len()
    }

    fn reach_forward(&self, src: EntityId) -> Vec<QueryIterator> {
        self.depth_first_search(src, Traversal::Forward)
    }

    fn reach_backward(&self, src: EntityId) -> Vec<QueryIterator> {
        self.depth_first_search(src, Traversal::Backward)
    }

    fn reach_forward_until(&self, src: EntityId, tgt: EntityId) -> Option<QueryIterator> {
        let reach = self.reach_forward(src);
        let path = reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .cloned()
            .collect_vec();
        if path.len() > 0 {
            Some((self, path).into())
        } else {
            None
        }
    }

    fn reach_backward_until(&self, src: EntityId, tgt: EntityId) -> Option<QueryIterator> {
        let reach = self.reach_backward(src);
        let path = reach
            .iter()
            .flatten()
            .filter(|t| *t == &tgt)
            .cloned()
            .collect_vec();
        if path.len() > 0 {
            Some((self, path).into())
        } else {
            None
        }
    }

    fn are_reachable(&self, src: EntityId, tgt: EntityId) -> bool {
        self.reach_forward_until(src, tgt).is_some()
    }

    fn depth_first_search(&self, src: EntityId, traversal: Traversal) -> Vec<QueryIterator> {
        fn depth_first_search_rec(
            traversal: &Traversal,
            engine_state: &Arc<EngineState>,
            results: &mut Vec<QueryIterator>,
            freelist: &mut VecDeque<EntityId>,
            finished: &mut HashSet<EntityId>,
            history: &mut Vec<EntityId>,
        ) {
            while let Some(current_node) = freelist.pop_back() {
                finished.insert(current_node);
                history.push(current_node);

                let neighbors = match traversal {
                    Traversal::Forward => engine_state.get_forward_neighbors(current_node),
                    Traversal::Backward => engine_state.get_backward_neighbors(current_node),
                    Traversal::Both => engine_state.get_neighbors(current_node),
                }
                .into_iter()
                .cloned()
                .collect_vec();
                if neighbors.is_empty() {
                    results.push((engine_state, history.clone()).into());
                } else {
                    for neighbor in neighbors {
                        if !finished.contains(&neighbor) {
                            freelist.push_back(neighbor);
                            depth_first_search_rec(
                                traversal,
                                engine_state,
                                results,
                                freelist,
                                finished,
                                history,
                            );
                            freelist.pop_back();
                        } else {
                            results.push((engine_state, history.clone()).into());
                            history.pop();
                        }
                    }
                }

                if let Some(popped) = history.pop() {
                    finished.remove(&popped);
                }
            }
        }

        let mut results: Vec<QueryIterator> = vec![];
        let mut freelist = VecDeque::default();
        let mut finished = HashSet::new();
        let mut history = vec![];
        freelist.push_back(src);

        depth_first_search_rec(
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
    use crate::{
        internals::{engine_state::EngineState, lifecycle::Lifecycle},
        layers::traversing::Traversing,
    };

    #[test]
    fn test_simple_reachability() {
        let engine_state = EngineState::new();

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
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        let y = engine_state
            .create_arrow(&b, &d, "Arrow".into(), vec![])
            .unwrap();
        let v = engine_state
            .create_arrow(&b, &d, "Arrow".into(), vec![])
            .unwrap();
        let z = engine_state
            .create_arrow(&d, &e, "Arrow".into(), vec![])
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
