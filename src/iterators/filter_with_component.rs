use std::sync::Arc;

use array_tool::vec::Shift;

use crate::internals::{Mosaic, Tile, WithMosaic, S32};

pub struct FilterComponentIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl FilterComponentIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>, component: S32) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        FilterComponentIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| t.component == component).collect(),
        }
    }
}

impl WithMosaic for FilterComponentIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for FilterComponentIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait FilterWithComponent: Iterator {
    fn filter_component(self, component: &str) -> FilterComponentIterator;
}

pub trait FilterComponentExtension: Iterator {
    fn filter_component_with(self, mosaic: Arc<Mosaic>, comp: &str) -> FilterComponentIterator;
}

impl<I> FilterWithComponent for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn filter_component(self, component: &str) -> FilterComponentIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        FilterComponentIterator::new(self, mosaic, component.into())
    }
}

impl<I> FilterComponentExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn filter_component_with(
        self,
        mosaic: Arc<Mosaic>,
        component: &str,
    ) -> FilterComponentIterator {
        FilterComponentIterator::new(self, mosaic, component.into())
    }
}
