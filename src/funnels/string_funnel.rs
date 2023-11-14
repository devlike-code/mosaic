use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use itertools::Itertools;

use crate::{
    internals::byte_utilities::FromByteArray,
    iterators::{
        filter_with_component::FilterWithComponent, get_dependent_tiles::GetDependentTilesExtension,
    },
};
use crate::{
    internals::{EntityId, Mosaic, MosaicCRUD, Tile, Value, B128},
    iterators::get_extensions::GetExtensions,
};

use crate::{
    internals::get_entities::GetEntitiesExtension,
    iterators::{get_dependent_tiles::GetDependentTiles, get_objects::GetObjects},
};

pub trait StringFunnel {
    /// Hash a string into an entity identifier (basic hash string helper function)
    fn hash_string(str: &str) -> EntityId;
    /// Creates a string object and attached outgoing properties returning the same id
    /// if called multiple times, without creating new morphisms
    fn create_string_object(&self, str: &str) -> anyhow::Result<Tile>;
    /// Recovers the string value from the entity by joining together all of the string parts
    fn recover_string(&self, tile: &Tile) -> Option<String>;
    /// Checks whether a string exists in the system
    fn string_exists(&self, str: &str) -> bool;
    /// Deletes the string from the system (will be reconstructed in the same way next time)
    fn delete_string(&self, str: &str);
}

impl StringFunnel for Arc<Mosaic> {
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

        let tile = self.new_specific_object(str_hash, "String".into())?;
        for part in split_str_into_parts(str, 128) {
            let mut ext = self.new_extension(&str_hash, "String".into());
            ext["self"] = Value::B128(B128::from_byte_array(part.as_bytes()));
            self.commit(&ext)?;
        }

        Ok(tile)
    }

    fn recover_string(&self, tile: &Tile) -> Option<String> {
        if !self.tile_exists(tile.id) {
            None
        } else {
            let parts = tile
                .iter_with(self)
                .get_dependents()
                .get_extensions()
                .filter_component("String")
                .flat_map(|t| t["self"].as_b128())
                .collect_vec();

            Some(String::from_utf8_lossy(&parts).to_string())
        }
    }

    fn string_exists(&self, str: &str) -> bool {
        let str_hash = Self::hash_string(str);
        self.tile_exists(str_hash)
    }

    fn delete_string(&self, str: &str) {
        todo!()
    }
}
