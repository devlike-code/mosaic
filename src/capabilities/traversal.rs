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
    internals::{get_tiles::GetTilesIterator, Mosaic, Tile, TileGetById, WithMosaic},
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

    pub fn depth_first_search(&self, from: &Tile) -> Vec<Vec<Tile>> {
        let mut result = Vec::new();
        let mut queue = VecDeque::new();
        queue.push_back((vec![], HashSet::new(), from.id));

        while let Some((mut trek, visited, current_id)) = queue.pop_front() {
            let current = self.mosaic.get(current_id).unwrap();
            // println!(
            //     "Trek: {:?}, Visited: {:?}, Current: {:?}",
            //     trek, visited, current
            // );
            trek.push(current.id);

            let neighbors = self.get_forward_neighbors(&current).collect_vec();
            // println!("Neighbors of {:?}: {:?}", current, neighbors);

            if !neighbors.is_empty() {
                let mut recursive = false;
                for neighbor in neighbors {
                    if !visited.contains(&neighbor.id) {
                        recursive = true;
                        let mut next_visited = visited.clone();
                        next_visited.insert(current.id);
                        queue.push_back((trek.clone(), next_visited, neighbor.id));
                    }
                }

                if !recursive {
                    // println!("RESULT FOUND: {:?}", trek);
                    result.push(trek.clone());
                }
            } else {
                // println!("RESULT FOUND: {:?}", trek);
                result.push(trek.clone());
            }

            // println!("QUEUE: {:?}", queue);
        }

        result
            .into_iter()
            .map(|path| self.mosaic.get_tiles(path).collect_vec())
            .collect_vec()
    }

    pub fn get_forward_paths(&self, from: &Tile) -> Vec<Vec<Tile>> {
        self.depth_first_search(from)
    }

    pub fn get_forward_path_between(&self, src: &Tile, tgt: &Tile) -> Option<Vec<Tile>> {
        let reach = self.get_forward_paths(src);
        let path = reach
            .into_iter()
            .flatten()
            .filter(|t| t == tgt)
            .collect_vec();

        if !path.is_empty() {
            Some(path)
        } else {
            None
        }
    }

    pub fn are_reachable(&self, src: &Tile, tgt: &Tile) -> bool {
        self.get_forward_path_between(src, tgt).is_some()
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
