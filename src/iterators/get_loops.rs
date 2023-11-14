use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetLoopsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetLoopsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetLoopsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_loop()).collect(),
        }
    }
}

impl WithMosaic for GetLoopsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetLoopsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetLoops: Iterator {
    fn get_loops(self) -> GetLoopsIterator;
}

pub trait GetLoopsExtension: Iterator {
    fn get_loops_with(self, mosaic: Arc<Mosaic>) -> GetLoopsIterator;
}

impl<I> GetLoops for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_loops(self) -> GetLoopsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetLoopsIterator::new(self, mosaic)
    }
}

impl<I> GetLoopsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_loops_with(self, mosaic: Arc<Mosaic>) -> GetLoopsIterator {
        GetLoopsIterator::new(self, mosaic)
    }
}
