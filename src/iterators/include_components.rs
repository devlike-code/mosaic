use std::sync::Arc;

use array_tool::vec::Shift;
use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic, S32};

pub struct IncludeComponentsIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl IncludeComponentsIterator {
    pub(crate) fn new<I>(iter: I, mosaic: Arc<Mosaic>, components: &[S32]) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        IncludeComponentsIterator {
            mosaic: Arc::clone(&mosaic),
            items: iter.filter(|t| components.contains(&t.component)).collect(),
        }
    }
}

impl WithMosaic for IncludeComponentsIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for IncludeComponentsIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.shift()
    }
}

pub trait IncludeComponents: Iterator {
    fn include_components(self, component: &[&str]) -> IncludeComponentsIterator;
}

pub trait IncludeComponentsExtension: Iterator {
    fn include_components_with(
        self,
        mosaic: Arc<Mosaic>,
        comp: &[&str],
    ) -> IncludeComponentsIterator;
}

impl<I> IncludeComponents for I
where
    I: Iterator<Item = Tile> + WithMosaic,
{
    fn include_components(self, components: &[&str]) -> IncludeComponentsIterator {
        let mosaic = Arc::clone(&self.get_mosaic());
        IncludeComponentsIterator::new(
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

impl<I> IncludeComponentsExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn include_components_with(
        self,
        mosaic: Arc<Mosaic>,
        components: &[&str],
    ) -> IncludeComponentsIterator {
        IncludeComponentsIterator::new(
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
