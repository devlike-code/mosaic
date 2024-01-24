use std::{collections::HashMap, sync::Arc};

use array_tool::vec::Uniq;
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
    fn remove_components(&self, target: &Tile, component: &str);

    fn match_archetype(&self, target: &Tile, components: &[&str]) -> bool {
        components
            .iter()
            .all(|c| self.get_component(target, c).is_some())
    }

    fn get_archetype(&self, target: &Tile, components: &[&str]) -> HashMap<String, Tile> {
        HashMap::from_iter(
            components
                .iter()
                .map(|c| (c.to_string(), self.get_component(target, c).unwrap())),
        )
    }

    fn get_full_archetype(&self, target: &Tile) -> HashMap<String, Vec<Tile>> {
        let mut types: Vec<String> = vec![target.component.into()];
        for desc in target.iter().get_descriptors() {
            types.push(desc.component.into());
        }
        types = types.unique();

        self.get_archetypes(target, types.as_slice())
    }

    fn get_archetypes<S: AsRef<str>>(
        &self,
        target: &Tile,
        components: &[S],
    ) -> HashMap<String, Vec<Tile>> {
        HashMap::from_iter(components.iter().map(|c| {
            (
                c.as_ref().to_string(),
                self.get_components(target, c.as_ref()),
            )
        }))
    }
}

pub trait ArchetypeSubject {
    fn get_component(&self, component: &str) -> Option<Tile>;
    fn get_components(&self, component: &str) -> Vec<Tile>;
    fn add_component(&self, component: &str, data: Vec<(S32, Value)>) -> Tile;
    fn remove_components(&self, component: &str);
    fn match_archetype(&self, components: &[&str]) -> bool;
    fn get_full_archetype(&self) -> HashMap<String, Vec<Tile>>;
    fn get_archetype(&self, components: &[&str]) -> HashMap<String, Tile>;
    fn get_archetypes(&self, components: &[&str]) -> HashMap<String, Vec<Tile>>;
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

    fn remove_components(&self, target: &Tile, component: &str) {
        target
            .iter()
            .get_dependents()
            .include_component(component)
            .delete();
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
        println!("{:?}", data);
        self.mosaic.add_component(self, component, data)
    }

    fn remove_components(&self, component: &str) {
        self.mosaic.remove_components(self, component)
    }

    fn match_archetype(&self, components: &[&str]) -> bool {
        self.mosaic.match_archetype(self, components)
    }

    fn get_full_archetype(&self) -> HashMap<String, Vec<Tile>> {
        self.mosaic.get_full_archetype(self)
    }

    fn get_archetype(&self, components: &[&str]) -> HashMap<String, Tile> {
        self.mosaic.get_archetype(self, components)
    }

    fn get_archetypes(&self, components: &[&str]) -> HashMap<String, Vec<Tile>> {
        self.mosaic.get_archetypes(self, components)
    }
}
