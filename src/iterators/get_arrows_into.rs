use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetArrowsIntoIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetArrowsIntoIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        let tile_storage = mosaic.tile_registry.lock().unwrap();
        let mut result = vec![];
        for item in iter {
            let id = item.id;
            result.push(
                mosaic
                    .dependent_ids_map
                    .lock()
                    .unwrap()
                    .get_all(&item.id)
                    .filter_map(|id| tile_storage.get(id))
                    .filter(|tile| tile.is_arrow())
                    .filter(|tile| tile.target_id() == id)
                    .cloned()
                    .collect_vec(),
            )
        }

        GetArrowsIntoIterator {
            mosaic: Arc::clone(&mosaic),
            items: result.into_iter().flatten().collect_vec(),
        }
    }
}

impl WithMosaic for GetArrowsIntoIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetArrowsIntoIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetArrowsIntoTiles: Iterator {
    fn get_arrows_into(self) -> GetArrowsIntoIterator;
}

pub trait GetArrowsIntoTilesExtension: Iterator {
    fn get_arrows_into_with(self, mosaic: Arc<Mosaic>) -> GetArrowsIntoIterator;
}

impl<I> GetArrowsIntoTiles for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_arrows_into(self) -> GetArrowsIntoIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetArrowsIntoIterator::new(self, mosaic)
    }
}

impl<I> GetArrowsIntoTilesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_arrows_into_with(self, mosaic: Arc<Mosaic>) -> GetArrowsIntoIterator {
        GetArrowsIntoIterator::new(self, mosaic)
    }
}
