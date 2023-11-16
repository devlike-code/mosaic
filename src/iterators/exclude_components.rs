use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic, S32};

pub struct ExcludeComponentsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl ExcludeComponentsIterator {
    pub(crate) fn new<I>(iter: I, mosaic: Arc<Mosaic>, components: &[S32]) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        ExcludeComponentsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter
                .filter(|t| !components.contains(&t.component))
                .collect(),
        }
    }
}

impl WithMosaic for ExcludeComponentsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for ExcludeComponentsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait ExcludeComponents: Iterator {
    fn exclude_components(self, component: &[&str]) -> ExcludeComponentsIterator;
}

pub trait ExcludeComponentsExtension: Iterator {
    fn exclude_components_with(
        self,
        mosaic: Arc<Mosaic>,
        comp: &[&str],
    ) -> ExcludeComponentsIterator;
}

impl<I> ExcludeComponents for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn exclude_components(self, components: &[&str]) -> ExcludeComponentsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        ExcludeComponentsIterator::new(
            self,
            mosaic,
            components
                .iter()
                .map(|&c| c.into())
                .collect_vec()
                .as_slice(),
        )
    }
}

impl<I> ExcludeComponentsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn exclude_components_with(
        self,
        mosaic: Arc<Mosaic>,
        components: &[&str],
    ) -> ExcludeComponentsIterator {
        ExcludeComponentsIterator::new(
            self,
            mosaic,
            components
                .iter()
                .map(|&c| c.into())
                .collect_vec()
                .as_slice(),
        )
    }
}
