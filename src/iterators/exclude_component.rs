use std::sync::Arc;

use crate::internals::{Mosaic, Tile, WithMosaic};

use super::exclude_components::{
    ExcludeComponents, ExcludeComponentsExtension, ExcludeComponentsIterator,
};

pub trait ExcludeComponent: ExcludeComponents {
    fn exclude_component(self, component: &str) -> ExcludeComponentsIterator;
}

pub trait ExcludeComponentExtension: ExcludeComponentsExtension {
    fn exclude_component_with(self, mosaic: Arc<Mosaic>, comp: &str) -> ExcludeComponentsIterator;
}

impl<I> ExcludeComponent for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn exclude_component(self, component: &str) -> ExcludeComponentsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        ExcludeComponentsIterator::new(self, mosaic, &[component.into()])
    }
}

impl<I> ExcludeComponentExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn exclude_component_with(
        self,
        mosaic: Arc<Mosaic>,
        component: &str,
    ) -> ExcludeComponentsIterator {
        ExcludeComponentsIterator::new(self, mosaic, &[component.into()])
    }
}
