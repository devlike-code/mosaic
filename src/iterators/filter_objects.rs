use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct FilterObjectsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl FilterObjectsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        FilterObjectsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_object()).collect(),
        }
    }
}

impl WithMosaic for FilterObjectsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for FilterObjectsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait FilterObjects: Iterator {
    fn get_objects(self) -> FilterObjectsIterator;
}

pub trait FilterObjectsExtension: Iterator {
    fn get_objects_with(self, mosaic: Arc<Mosaic>) -> FilterObjectsIterator;
}

impl<I> FilterObjects for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_objects(self) -> FilterObjectsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        FilterObjectsIterator::new(self, mosaic)
    }
}

impl<I> FilterObjectsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_objects_with(self, mosaic: Arc<Mosaic>) -> FilterObjectsIterator {
        FilterObjectsIterator::new(self, mosaic)
    }
}
