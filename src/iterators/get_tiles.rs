use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetTilesIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetTilesIterator {
    pub fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetTilesIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.collect(),
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

impl<I> GetTiles for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_tiles(self) -> GetTilesIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetTilesIterator::new(self, mosaic)
    }
}

impl<I> GetTilesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_tiles_with(self, mosaic: Arc<Mosaic>) -> GetTilesIterator {
        GetTilesIterator::new(self, mosaic)
    }
}
