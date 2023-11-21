use std::sync::Arc;

use crate::internals::{Tile, WithMosaic};

use super::include_components::{
    IncludeComponents, IncludeComponentsIterator,
};

pub trait IncludeComponent: IncludeComponents {
    fn include_component(self, component: &str) -> IncludeComponentsIterator;
}


impl<I> IncludeComponent for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn include_component(self, component: &str) -> IncludeComponentsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        IncludeComponentsIterator::new(self, mosaic, &[component.into()])
    }
}