use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetDependentsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetDependentsIterator {
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

        GetDependentsIterator {
            mosaic: Arc::clone(&mosaic),
            items: result.into_iter().flatten().collect_vec(),
        }
    }
}

impl WithMosaic for GetDependentsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetDependentsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetDependentTiles: Iterator {
    fn get_dependents(self) -> GetDependentsIterator;
}

pub trait GetDependentTilesExtension: Iterator {
    fn get_dependents_with(self, mosaic: Arc<Mosaic>) -> GetDependentsIterator;
}

impl<I> GetDependentTiles for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_dependents(self) -> GetDependentsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetDependentsIterator::new(self, mosaic)
    }
}

impl<I> GetDependentTilesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_dependents_with(self, mosaic: Arc<Mosaic>) -> GetDependentsIterator {
        GetDependentsIterator::new(self, mosaic)
    }
}
