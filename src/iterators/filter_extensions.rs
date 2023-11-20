use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct FilterExtensionsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl FilterExtensionsIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        FilterExtensionsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.is_extension()).collect(),
        }
    }
}

impl WithMosaic for FilterExtensionsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for FilterExtensionsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait FilterExtensions: Iterator {
    fn filter_extensions(self) -> FilterExtensionsIterator;
}

pub trait FilterExtensionsExtension: Iterator {
    fn get_extensions_with(self, mosaic: Arc<Mosaic>) -> FilterExtensionsIterator;
}

impl<I> FilterExtensions for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn filter_extensions(self) -> FilterExtensionsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        FilterExtensionsIterator::new(self, mosaic)
    }
}

impl<I> FilterExtensionsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_extensions_with(self, mosaic: Arc<Mosaic>) -> FilterExtensionsIterator {
        FilterExtensionsIterator::new(self, mosaic)
    }
}
