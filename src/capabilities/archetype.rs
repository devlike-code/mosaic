use std::sync::Arc;

use itertools::Itertools;

use crate::{
    internals::{Mosaic, MosaicCRUD, Tile, Value, S32},
    iterators::{
        component_selectors::ComponentSelectors, tile_deletion::TileDeletion,
        tile_getters::TileGetters,
    },
};

pub trait Archetype {
    fn get_component(&self, target: &Tile, component: &str) -> Option<Tile>;
    fn get_components(&self, target: &Tile, component: &str) -> Vec<Tile>;
    fn add_component(&self, target: &Tile, component: &str, data: Vec<(S32, Value)>) -> Tile;
    fn remove_component(&self, target: &Tile, component: &str) -> Option<Tile>;
    fn remove_components(&self, target: &Tile, component: &str) -> Vec<Tile>;
}

pub trait ArchetypeSubject {
    fn get_component(&self, component: &str) -> Option<Tile>;
    fn get_components(&self, component: &str) -> Vec<Tile>;
    fn add_component(&self, component: &str, data: Vec<(S32, Value)>) -> Tile;
    fn remove_component(&self, component: &str) -> Option<Tile>;
    fn remove_components(&self, component: &str) -> Vec<Tile>;
}

impl Archetype for Arc<Mosaic> {
    fn get_component(&self, target: &Tile, component: &str) -> Option<Tile> {
        if target.component == component.into() {
            return Some(target.clone());
        }

        let comps = target
            .iter()
            .get_dependents()
            .include_component(component)
            .collect_vec();

        comps.first().cloned()
    }

    fn get_components(&self, target: &Tile, component: &str) -> Vec<Tile> {
        let mut result = vec![];
        if target.component == component.into() {
            result.push(target.clone());
        }

        let comps = target
            .iter()
            .get_dependents()
            .include_component(component)
            .collect_vec();

        result.extend(comps);
        result
    }

    fn add_component(&self, target: &Tile, component: &str, data: Vec<(S32, Value)>) -> Tile {
        self.new_descriptor(target, component, data)
    }

    fn remove_component(&self, target: &Tile, component: &str) -> Option<Tile> {
        let comps = target.iter().get_dependents().include_component(component);
        let collected = comps.collect_vec().first().cloned();

        collected
            .iter()
            .map(|t| {
                t.iter().delete();
                t
            })
            .collect_vec();

        collected
    }

    fn remove_components(&self, target: &Tile, component: &str) -> Vec<Tile> {
        let collected = target
            .iter()
            .get_dependents()
            .include_component(component)
            .collect_vec();

        collected.iter().map(|t| t.iter().delete()).collect_vec();
        collected
    }
}

impl ArchetypeSubject for Tile {
    fn get_component(&self, component: &str) -> Option<Tile> {
        self.mosaic.get_component(self, component)
    }

    fn get_components(&self, component: &str) -> Vec<Tile> {
        self.mosaic.get_components(self, component)
    }

    fn add_component(&self, component: &str, data: Vec<(S32, Value)>) -> Tile {
        self.mosaic.add_component(self, component, data)
    }

    fn remove_component(&self, component: &str) -> Option<Tile> {
        self.mosaic.remove_component(self, component)
    }

    fn remove_components(&self, component: &str) -> Vec<Tile> {
        self.mosaic.remove_components(self, component)
    }
}
