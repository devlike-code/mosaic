use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct GetSourcesIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetSourcesIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        GetSourcesIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.flat_map(|t| mosaic.get(t.source_id())).collect(),
        }
    }
}

impl WithMosaic for GetSourcesIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for GetSourcesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait GetSources: Iterator {
    fn get_sources(self) -> GetSourcesIterator;
}

pub trait GetSourcesExtension: Iterator {
    fn get_sources_with(self, mosaic: Arc<Mosaic>) -> GetSourcesIterator;
}

impl<I> GetSources for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn get_sources(self) -> GetSourcesIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        GetSourcesIterator::new(self, mosaic)
    }
}

impl<I> GetSourcesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_sources_with(self, mosaic: Arc<Mosaic>) -> GetSourcesIterator {
        GetSourcesIterator::new(self, mosaic)
    }
}
