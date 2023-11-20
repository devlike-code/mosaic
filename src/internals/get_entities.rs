use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetEntitiesIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetEntitiesIterator {
    pub(crate) fn new(mosaic: Arc<Mosaic>) -> Self {
        GetEntitiesIterator {
            mosaic: Arc::clone(&mosaic),
            items: mosaic
                .tile_registry
                .lock()
                .unwrap()
                .values()
                .cloned()
                .collect_vec(),
        }
    }
}

impl WithMosaic for GetEntitiesIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetEntitiesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetEntities: Iterator {
    fn get_entities(self) -> GetEntitiesIterator;
}

pub trait GetEntitiesExtension: Iterator {
    fn get_entities_with(self, mosaic: Arc<Mosaic>) -> GetEntitiesIterator;
}

impl<I> GetEntities for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_entities(self) -> GetEntitiesIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetEntitiesIterator::new(mosaic)
    }
}

impl<I> GetEntitiesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_entities_with(self, mosaic: Arc<Mosaic>) -> GetEntitiesIterator {
        GetEntitiesIterator::new(mosaic)
    }
}
