use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetObjectsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetObjectsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetObjectsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_object()).collect(),
        }
    }
}

impl WithMosaic for GetObjectsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetObjectsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetObjects: Iterator {
    fn get_objects(self) -> GetObjectsIterator;
}

pub trait GetObjectsExtension: Iterator {
    fn get_objects_with(self, mosaic: Arc<Mosaic>) -> GetObjectsIterator;
}

impl<I> GetObjects for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_objects(self) -> GetObjectsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetObjectsIterator::new(self, mosaic)
    }
}

impl<I> GetObjectsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_objects_with(self, mosaic: Arc<Mosaic>) -> GetObjectsIterator {
        GetObjectsIterator::new(self, mosaic)
    }
}
