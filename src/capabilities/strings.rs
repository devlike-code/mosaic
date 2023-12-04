use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    internals::{par, MosaicIO},
    iterators::{component_selectors::ComponentSelectors, tile_getters::TileGetters},
};
use crate::{
    internals::{EntityId, Mosaic, MosaicCRUD, Tile},
    iterators::tile_filters::TileFilters,
};

pub trait StringCapability {
    fn hash_string(str: &str) -> EntityId;
    fn create_string_object(&self, str: &str) -> anyhow::Result<Tile>;
    fn get_string_value(&self, tile: &Tile) -> Option<String>;
    fn string_exists(&self, str: &str) -> bool;
    fn delete_string(&self, str: &str);
}

impl StringCapability for Arc<Mosaic> {
    fn hash_string(str: &str) -> EntityId {
        let mut hasher = DefaultHasher::new();
        str.hash(&mut hasher);
        hasher.finish().try_into().unwrap()
    }

    fn create_string_object(&self, str: &str) -> anyhow::Result<Tile> {
        fn split_str_into_parts(input: &str, part_size: usize) -> impl Iterator<Item = &str> {
            input
                .char_indices()
                .step_by(part_size)
                .map(move |(start, _)| &input[start..(start + part_size).min(input.len())])
        }

        let str_hash = Self::hash_string(str);

        let tile = self.new_specific_object(str_hash, "String")?;

        assert!(self.is_tile_valid(&str_hash));

        for part in split_str_into_parts(str, 128) {
            self.new_extension(&str_hash, "String", par(part.as_bytes()));
        }

        Ok(tile)
    }

    fn get_string_value(&self, tile: &Tile) -> Option<String> {
        if !self.is_tile_valid(tile) {
            None
        } else {
            let parts = tile
                .clone()
                .into_iter()
                .get_dependents()
                .filter_extensions()
                .include_component("String")
                .flat_map(|t| t.get("self").as_s128())
                .collect_vec();

            Some(String::from_utf8_lossy(&parts).to_string())
        }
    }

    fn string_exists(&self, str: &str) -> bool {
        let str_hash = Self::hash_string(str);
        self.is_tile_valid(&str_hash)
    }

    fn delete_string(&self, str: &str) {
        let str_hash = Self::hash_string(str);
        self.delete_tile(str_hash);
    }
}
