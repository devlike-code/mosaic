use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

use super::EntityId;

pub struct GetTilesIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetTilesIterator {
    pub fn new<I: Iterator<Item = Tile>>(iter: I, mosaic: Arc<Mosaic>) -> Self {
        GetTilesIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.collect(),
        }
    }

    pub fn new_from_ids<I: Iterator<Item = EntityId>>(iter: I, mosaic: Arc<Mosaic>) -> Self {
        GetTilesIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.into_iter().flat_map(|id| mosaic.get(id)).collect(),
        }
    }
}

impl WithMosaic for GetTilesIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetTiles: Iterator {
    fn get_tiles(self) -> GetTilesIterator;
}

pub trait GetTilesExtension: Iterator {
    fn get_tiles_with(self, mosaic: Arc<Mosaic>) -> GetTilesIterator;
}

impl<I: Iterator<Item = Tile> + WithMosaic> GetTiles for I {
    fn get_tiles(self) -> GetTilesIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetTilesIterator::new(self, mosaic)
    }
}

impl<I: Iterator<Item = Tile>> GetTilesExtension for I {
    fn get_tiles_with(self, mosaic: Arc<Mosaic>) -> GetTilesIterator {
        GetTilesIterator::new(self, mosaic)
    }
}
