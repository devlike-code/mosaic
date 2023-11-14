use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetExtensionsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetExtensionsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetExtensionsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_extension()).collect(),
        }
    }
}

impl WithMosaic for GetExtensionsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetExtensionsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetExtensions: Iterator {
    fn get_extensions(self) -> GetExtensionsIterator;
}

pub trait GetExtensionsExtension: Iterator {
    fn get_extensions_with(self, mosaic: Arc<Mosaic>) -> GetExtensionsIterator;
}

impl<I> GetExtensions for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_extensions(self) -> GetExtensionsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetExtensionsIterator::new(self, mosaic)
    }
}

impl<I> GetExtensionsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_extensions_with(self, mosaic: Arc<Mosaic>) -> GetExtensionsIterator {
        GetExtensionsIterator::new(self, mosaic)
    }
}
