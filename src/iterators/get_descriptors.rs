use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetDescriptorsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetDescriptorsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetDescriptorsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_descriptor()).collect(),
        }
    }
}

impl WithMosaic for GetDescriptorsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetDescriptorsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetDescriptors: Iterator {
    fn get_descriptors(self) -> GetDescriptorsIterator;
}

pub trait GetDescriptorsExtension: Iterator {
    fn get_descriptors_with(self, mosaic: Arc<Mosaic>) -> GetDescriptorsIterator;
}

impl<I> GetDescriptors for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_descriptors(self) -> GetDescriptorsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetDescriptorsIterator::new(self, mosaic)
    }
}

impl<I> GetDescriptorsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_descriptors_with(self, mosaic: Arc<Mosaic>) -> GetDescriptorsIterator {
        GetDescriptorsIterator::new(self, mosaic)
    }
}
