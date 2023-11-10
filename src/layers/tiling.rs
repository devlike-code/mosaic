use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::internals::{
    mosaic_engine::MosaicEngine, query_iterator::QueryIterator, Block, DataBrick, EngineState,
    EntityId, Tile,
};

use super::{accessing::Accessing, querying::Querying};

pub(crate) fn tile_from_brick_data(mosaic: &Arc<MosaicEngine>, brick: &DataBrick) -> Tile {
    (mosaic, brick).into()
}

pub trait Tiling {
    fn get_blocks(&self, selection: Option<QueryIterator>) -> HashMap<EntityId, Block>;
    fn get_tile(&self, brick: EntityId) -> Option<Tile>;
}

impl Tiling for Arc<MosaicEngine> {
    fn get_blocks(&self, filter: Option<QueryIterator>) -> HashMap<EntityId, Block> {
        let selection = if filter.is_none() {
            self.engine_state.query_access().get()
        } else {
            filter.unwrap().as_slice().into_iter().fold(
                (Arc::clone(&self.engine_state), vec![]).into(),
                |old: QueryIterator, &f| old.union(self.engine_state.get_edges(f)),
            )
        };

        let tiles = selection
            .as_slice()
            .into_iter()
            .flat_map(|id| self.engine_state.get_brick(*id))
            .map(|brick| {
                let tile: Tile = (self, &brick).into();
                tile
            })
            .collect_vec();

        let mut result = HashMap::new();

        for tile in tiles {
            let id = tile.id();
            if !result.contains_key(&id) {
                result.insert(id, Block::new());
            }
            result
                .get_mut(&id)
                .unwrap()
                .extend(vec![tile.clone()].into());

            let (source, target) = tile.get_endpoints();
            let (source, target) = (source.id(), target.id());
            if !result.contains_key(&source) {
                result.insert(source, Block::new());
            }
            result
                .get_mut(&source)
                .unwrap()
                .extend(vec![tile.clone()].into());

            if !result.contains_key(&target) {
                result.insert(target, Block::new());
            }
            result
                .get_mut(&target)
                .unwrap()
                .extend(vec![tile.polarize_towards(target)].into());
        }
        result
    }

    fn get_tile(&self, brick: EntityId) -> Option<Tile> {
        let brick = self.engine_state.get_brick(brick)?;
        Some(tile_from_brick_data(self, &brick))
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod tiling_testing {}
