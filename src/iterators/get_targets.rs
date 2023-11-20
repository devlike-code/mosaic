use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetTargetsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetTargetsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetTargetsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.flat_map(|t| mosaic.get(t.target_id())).collect(),
        }
    }
}

impl WithMosaic for GetTargetsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetTargetsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetTargets: Iterator {
    fn get_targets(self) -> GetTargetsIterator;
}

pub trait GetTargetsExtension: Iterator {
    fn get_targets_with(self, mosaic: Arc<Mosaic>) -> GetTargetsIterator;
}

impl<I> GetTargets for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_targets(self) -> GetTargetsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetTargetsIterator::new(self, mosaic)
    }
}

impl<I> GetTargetsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_targets_with(self, mosaic: Arc<Mosaic>) -> GetTargetsIterator {
        GetTargetsIterator::new(self, mosaic)
    }
}
