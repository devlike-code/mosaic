use std::sync::Arc;

use crate::internals::{Mosaic, Tile, Value};

pub trait Archetype {
    fn get_component(&self, target: &Tile, component: &str) -> Option<Tile>;
    fn get_components(&self, target: &Tile, component: &str) -> Vec<Tile>;
    fn add_component(&self, target: &Tile, component: &str, data: Vec<Value>) -> Option<Tile>;
    fn remove_component(&self, target: &Tile, component: &str);
}

pub trait ArchetypeSubject {
    fn get_component(&self, component: &str) -> Option<Tile>;
    fn get_components(&self, component: &str) -> Vec<Tile>;
    fn add_component(&self, component: &str, data: Vec<Value>) -> Option<Tile>;
    fn remove_component(&self, component: &str);
}

impl Archetype for Arc<Mosaic> {
    fn get_component(&self, target: &Tile, component: &str) -> Option<Tile> {
        None
    }

    fn get_components(&self, target: &Tile, component: &str) -> Vec<Tile> {
        Vec::new()
    }

    fn add_component(&self, target: &Tile, component: &str, data: Vec<Value>) -> Option<Tile> {
        None
    }

    fn remove_component(&self, target: &Tile, component: &str) {}
}

impl ArchetypeSubject for Tile {
    fn get_component(&self, component: &str) -> Option<Tile> {
        self.mosaic.get_component(self, component)
    }

    fn get_components(&self, component: &str) -> Vec<Tile> {
        self.mosaic.get_components(self, component)
    }

    fn add_component(&self, component: &str, data: Vec<Value>) -> Option<Tile> {
        self.mosaic.add_component(self, component, data)
    }

    fn remove_component(&self, component: &str) {
        self.mosaic.remove_component(self, component)
    }
}
