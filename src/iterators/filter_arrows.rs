use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct FilterArrowsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl FilterArrowsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        FilterArrowsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_arrow()).collect(),
        }
    }
}

impl WithMosaic for FilterArrowsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for FilterArrowsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait FilterArrows: Iterator {
    fn filter_arrows(self) -> FilterArrowsIterator;
}

pub trait FilterArrowsExtension: Iterator {
    fn get_arrows_with(self, mosaic: Arc<Mosaic>) -> FilterArrowsIterator;
}

impl<I> FilterArrows for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn filter_arrows(self) -> FilterArrowsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        FilterArrowsIterator::new(self, mosaic)
    }
}

impl<I> FilterArrowsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_arrows_with(self, mosaic: Arc<Mosaic>) -> FilterArrowsIterator {
        FilterArrowsIterator::new(self, mosaic)
    }
}
