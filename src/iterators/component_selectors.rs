use std::vec::IntoIter;

use itertools::Itertools;

use crate::internals::Tile;

pub trait ComponentSelectors: Iterator {
    fn include_components(self, components: &[String]) -> IntoIter<Self::Item>;
    fn include_component(self, component: &str) -> IntoIter<Self::Item>;
    fn exclude_components(self, components: &[String]) -> IntoIter<Self::Item>;
    fn exclude_component(self, component: &str) -> IntoIter<Self::Item>;
}

impl<I> ComponentSelectors for I
where
    I: Iterator<Item = Tile>,
{
    fn include_components(self, components: &[String]) -> IntoIter<Self::Item> {
        let binding = components.iter().map(|c| c.as_str().into()).collect_vec();
        let components = binding.as_slice();

        self.filter(|t| components.contains(&t.component))
            .collect_vec()
            .into_iter()
    }

    fn include_component(self, component: &str) -> IntoIter<Self::Item> {
        self.include_components(&[component.to_string()])
    }

    fn exclude_components(self, components: &[String]) -> IntoIter<Self::Item> {
        let binding = components.iter().map(|c| c.as_str().into()).collect_vec();
        let components = binding.as_slice();

        self.filter(|t| !components.contains(&t.component))
            .collect_vec()
            .into_iter()
    }

    fn exclude_component(self, component: &str) -> IntoIter<Self::Item> {
        self.exclude_components(&[component.to_string()])
    }
}
