#[derive(Debug, Default, PartialEq, Clone)]
pub enum TraversalDirection {
    #[default]
    Forward,
    Backward,
    Both,
}

use std::{
    collections::{HashSet, VecDeque},
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    internals::{Mosaic, Tile, WithMosaic},
    iterators::{
        exclude_components::ExcludeComponents,
        get_arrows_from::GetArrowsFromTiles,
        get_arrows_into::GetArrowsIntoTiles,
        get_sources::{GetSourcesExtension, GetSourcesIterator},
        get_targets::{GetTargetsExtension, GetTargetsIterator},
        include_components::IncludeComponents,
    },
};

pub enum Traversal {
    Exclude { components: &'static [&'static str] },
    Include { components: &'static [&'static str] },
}

pub struct TraversalOperator {
    pub(crate) mosaic: Arc<Mosaic>,
    pub(crate) traversal: Traversal,
}

impl TraversalOperator {
    fn filter_traversal<I: Iterator<Item = Tile> + WithMosaic>(&self, iter: I) -> Vec<Tile> {
        match self.traversal {
            Traversal::Exclude { components } => iter.exclude_components(components).collect_vec(),
            Traversal::Include { components } => iter.include_components(components).collect_vec(),
        }
    }

    pub fn out_degree(&self, tile: &Tile) -> usize {
        self.filter_traversal(tile.iter_with(&self.mosaic).get_arrows_from())
            .len()
    }

    pub fn in_degree(&self, tile: &Tile) -> usize {
        self.filter_traversal(tile.iter_with(&self.mosaic).get_arrows_into())
            .len()
    }

    pub fn get_forward_neighbors(&self, tile: &Tile) -> GetTargetsIterator {
        self.filter_traversal(tile.iter_with(&self.mosaic).get_arrows_from())
            .into_iter()
            .get_targets_with(Arc::clone(&self.mosaic))
    }

    pub fn get_backward_neighbors(&self, tile: &Tile) -> GetSourcesIterator {
        self.filter_traversal(tile.iter_with(&self.mosaic).get_arrows_into())
            .into_iter()
            .get_sources_with(Arc::clone(&self.mosaic))
    }
    /*
        fn depth_first_search(&self, src: &Tile, direction: TraversalDirection) -> Vec<Vec<Tile>> {
            fn depth_first_search_rec(
                mosaic: Arc<Mosaic>,
                direction: &TraversalDirection,
                results: &mut Vec<Vec<Tile>>,
                freelist: &mut VecDeque<&Tile>,
                finished: &mut HashSet<&Tile>,
                history: &mut Vec<&Tile>,
            ) {
                while let Some(current_node) = freelist.pop_back() {
                    finished.insert(current_node);
                    history.push(current_node);

                    let neighbors = match direction {
                        TraversalDirection::Forward => {
                            engine_state.get_forward_neighbors(&current_node)
                        }
                        TraversalDirection::Backward => {
                            engine_state.get_backward_neighbors(&current_node)
                        }
                        TraversalDirection::Both => engine_state.get_neighbors(&current_node),
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
            let mut freelist: VecDeque<usize> = VecDeque::default();
            let mut finished = HashSet::new();
            let mut history = vec![];
            freelist.push_back(*src);

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
    */
}

pub trait Traverse {
    fn traverse(&self, traversal: Traversal) -> TraversalOperator;
}

impl Traverse for Arc<Mosaic> {
    fn traverse(&self, traversal: Traversal) -> TraversalOperator {
        TraversalOperator {
            mosaic: Arc::clone(self),
            traversal,
        }
    }
}

#[cfg(test)]
mod quick_test {
    use std::default;

    use itertools::Itertools;

    use crate::{
        capabilities::traversal::Traverse,
        internals::{Mosaic, MosaicCRUD},
    };

    use super::Traversal;

    #[test]
    fn test_neighborhoods() {
        let t = Traversal::Exclude {
            components: &["Parent", "Child"],
        };

        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let c = mosaic.new_object("DEBUG");
        let d = mosaic.new_object("DEBUG");

        /*
                      /----> b
           a ----parent----> c
                      \----> d

           a ----> b <----> c -----> d
        */
        mosaic.new_arrow(&a, &b, "Parent");
        mosaic.new_arrow(&a, &c, "Parent");
        mosaic.new_arrow(&a, &d, "Parent");
        mosaic.new_arrow(&a, &b, "DEBUG");
        mosaic.new_arrow(&b, &c, "DEBUG");
        mosaic.new_arrow(&c, &b, "DEBUG");
        mosaic.new_arrow(&c, &d, "DEBUG");

        let p = mosaic.traverse(t);
        assert_eq!(1, p.out_degree(&a));
        assert_eq!(0, p.in_degree(&a));

        assert_eq!(1, p.out_degree(&b));
        assert_eq!(2, p.in_degree(&b));

        let a_fwd_neighbors = p.get_forward_neighbors(&a).collect_vec();
        assert!(a_fwd_neighbors.contains(&b));

        let a_bwd_neighbors = p.get_backward_neighbors(&a).collect_vec();
        assert!(a_bwd_neighbors.is_empty());

        assert_eq!(None, p.get_forward_neighbors(&d).next());

        let c_fwd_neighbors = p.get_forward_neighbors(&c).collect_vec();
        assert!(c_fwd_neighbors.contains(&b));
        assert!(c_fwd_neighbors.contains(&d));

        let c_bwd_neighbors = p.get_backward_neighbors(&c).collect_vec();
        assert!(c_bwd_neighbors.contains(&b));

        let d_bwd_neighbors = p.get_backward_neighbors(&d).collect_vec();
        assert!(d_bwd_neighbors.contains(&c));

        //assert!(p.are_reachable(a, d));
        //println!(p.reach_forward(d));
    }
}
