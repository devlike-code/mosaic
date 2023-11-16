use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetArrowsFromIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetArrowsFromIterator {
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
                    .get_all(&id)
                    .filter_map(|id| tile_storage.get(id))
                    .filter(|tile| tile.is_arrow() &&  tile.source_id() == id)                  
                    .cloned()
                    .collect_vec(),
            )
        }

        GetArrowsFromIterator {
            mosaic: Arc::clone(&mosaic),
            items: result.into_iter().flatten().collect_vec(),
        }
    }
}

impl WithMosaic for GetArrowsFromIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetArrowsFromIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetArrowsFromTiles: Iterator {
    fn get_arrows_from(self) -> GetArrowsFromIterator;
}

pub trait GetArrowsFromTilesExtension: Iterator {
    fn get_arrows_from_with(self, mosaic: Arc<Mosaic>) -> GetArrowsFromIterator;
}

impl<I> GetArrowsFromTiles for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_arrows_from(self) -> GetArrowsFromIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetArrowsFromIterator::new(self, mosaic)
    }
}

impl<I> GetArrowsFromTilesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_arrows_from_with(self, mosaic: Arc<Mosaic>) -> GetArrowsFromIterator {
        GetArrowsFromIterator::new(self, mosaic)
    }
}
