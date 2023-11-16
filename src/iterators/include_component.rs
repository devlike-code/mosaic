use std::sync::Arc;

use crate::internals::{Mosaic, Tile, WithMosaic};

use super::include_components::{
    IncludeComponents, IncludeComponentsExtension, IncludeComponentsIterator,
};

pub trait IncludeComponent: IncludeComponents {
    fn include_component(self, component: &str) -> IncludeComponentsIterator;
}

pub trait IncludeComponentExtension: IncludeComponentsExtension {
    fn include_component_with(self, mosaic: Arc<Mosaic>, comp: &str) -> IncludeComponentsIterator;
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

impl<I> IncludeComponentExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn include_component_with(
        self,
        mosaic: Arc<Mosaic>,
        component: &str,
    ) -> IncludeComponentsIterator {
        IncludeComponentsIterator::new(self, mosaic, &[component.into()])
    }
}
