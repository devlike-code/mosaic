use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetArrowsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetArrowsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetArrowsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_arrow()).collect(),
        }
    }
}

impl WithMosaic for GetArrowsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetArrowsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetArrows: Iterator {
    fn get_arrows(self) -> GetArrowsIterator;
}

pub trait GetArrowsExtension: Iterator {
    fn get_arrows_with(self, mosaic: Arc<Mosaic>) -> GetArrowsIterator;
}

impl<I> GetArrows for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_arrows(self) -> GetArrowsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetArrowsIterator::new(self, mosaic)
    }
}

impl<I> GetArrowsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_arrows_with(self, mosaic: Arc<Mosaic>) -> GetArrowsIterator {
        GetArrowsIterator::new(self, mosaic)
    }
}
