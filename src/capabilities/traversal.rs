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
    internals::{get_tiles::GetTilesIterator, EntityId, Mosaic, Tile, TileGetById, WithMosaic},
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

    pub fn get_neighbors(&self, tile: &Tile) -> GetTilesIterator {
        let mut result = self.get_backward_neighbors(tile).collect_vec();
        result.extend(self.get_forward_neighbors(tile));
        GetTilesIterator::new(result.into_iter(), Arc::clone(&self.mosaic))
    }

    pub fn depth_first_search(&self, src: &Tile, direction: TraversalDirection) -> Vec<Vec<Tile>> {
        fn depth_first_search_rec(
            mosaic: &Arc<Mosaic>,
            operator: &TraversalOperator,
            direction: &TraversalDirection,
            results: &mut Vec<Vec<Tile>>,
            freelist: &mut VecDeque<EntityId>,
            finished: &mut HashSet<EntityId>,
            history: &mut Vec<EntityId>,
        ) {
            while let Some(current_id) = freelist.pop_back() {
                let current_node = mosaic.get(current_id).unwrap();
                finished.insert(current_node.id);
                history.push(current_node.id);

                let neighbors = match direction {
                    TraversalDirection::Forward => {
                        operator.get_forward_neighbors(&current_node).collect_vec()
                    }

                    TraversalDirection::Backward => {
                        operator.get_backward_neighbors(&current_node).collect_vec()
                    }

                    TraversalDirection::Both => operator.get_neighbors(&current_node).collect_vec(),
                };

                if neighbors.is_empty() {
                    results.push(mosaic.get_tiles(history.clone()).collect_vec());
                } else {
                    for neighbor in neighbors {
                        if !finished.contains(&neighbor.id) {
                            freelist.push_back(neighbor.id);
                            depth_first_search_rec(
                                mosaic, operator, direction, results, freelist, finished, history,
                            );
                            freelist.pop_back();
                        } else {
                            results.push(mosaic.get_tiles(history.clone()).collect_vec());
                            history.pop();
                        }
                    }
                }

                if let Some(popped) = history.pop() {
                    finished.remove(&popped);
                }
            }
        }

        let mut results: Vec<Vec<Tile>> = vec![];
        let mut freelist: VecDeque<usize> = VecDeque::default();
        let mut finished = HashSet::new();
        let mut history = vec![];
        freelist.push_back(src.id);

        depth_first_search_rec(
            &Arc::clone(&self.mosaic),
            self,
            &direction,
            &mut results,
            &mut freelist,
            &mut finished,
            &mut history,
        );
        results
    }
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
