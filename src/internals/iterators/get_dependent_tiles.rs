use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetDependentTilesIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetDependentTilesIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        let tile_storage = mosaic.tile_registry.lock().unwrap();
        let mut result = vec![];
        for item in iter {
            result.push(
                mosaic
                    .dependent_ids_map
                    .lock()
                    .unwrap()
                    .get_all(&item.id)
                    .filter_map(|id| tile_storage.get(id))
                    .cloned()
                    .collect_vec(),
            )
        }

        GetDependentTilesIterator {
            mosaic: Arc::clone(&mosaic),
            items: result.into_iter().flatten().collect_vec(),
        }
    }
}

impl WithMosaic for GetDependentTilesIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetDependentTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetDependentTiles: Iterator {
    fn get_dependent_tiles(self) -> GetDependentTilesIterator;
}

pub trait GetDependentTilesExtension: Iterator {
    fn get_dependent_tiles_with(self, mosaic: Arc<Mosaic>) -> GetDependentTilesIterator;
}

impl<I> GetDependentTiles for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_dependent_tiles(self) -> GetDependentTilesIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetDependentTilesIterator::new(self, mosaic)
    }
}

impl<I> GetDependentTilesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_dependent_tiles_with(self, mosaic: Arc<Mosaic>) -> GetDependentTilesIterator {
        GetDependentTilesIterator::new(self, mosaic)
    }
}
