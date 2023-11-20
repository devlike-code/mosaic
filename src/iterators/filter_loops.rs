use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct FilterLoopsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl FilterLoopsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        FilterLoopsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_loop()).collect(),
        }
    }
}

impl WithMosaic for FilterLoopsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for FilterLoopsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait FilterLoops: Iterator {
    fn get_loops(self) -> FilterLoopsIterator;
}

pub trait FilterLoopsExtension: Iterator {
    fn get_loops_with(self, mosaic: Arc<Mosaic>) -> FilterLoopsIterator;
}

impl<I> FilterLoops for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_loops(self) -> FilterLoopsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        FilterLoopsIterator::new(self, mosaic)
    }
}

impl<I> FilterLoopsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_loops_with(self, mosaic: Arc<Mosaic>) -> FilterLoopsIterator {
        FilterLoopsIterator::new(self, mosaic)
    }
}
