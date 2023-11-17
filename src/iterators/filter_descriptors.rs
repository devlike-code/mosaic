use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct FilterDescriptorsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl FilterDescriptorsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        FilterDescriptorsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_descriptor()).collect(),
        }
    }
}

impl WithMosaic for FilterDescriptorsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for FilterDescriptorsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait FilterDescriptors: Iterator {
    fn filter_descriptors(self) -> FilterDescriptorsIterator;
}

pub trait FilterDescriptorsExtension: Iterator {
    fn get_descriptors_with(self, mosaic: Arc<Mosaic>) -> FilterDescriptorsIterator;
}

impl<I> FilterDescriptors for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn filter_descriptors(self) -> FilterDescriptorsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        FilterDescriptorsIterator::new(self, mosaic)
    }
}

impl<I> FilterDescriptorsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_descriptors_with(self, mosaic: Arc<Mosaic>) -> FilterDescriptorsIterator {
        FilterDescriptorsIterator::new(self, mosaic)
    }
}
