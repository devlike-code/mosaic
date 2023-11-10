use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    sync::Arc,
};

use crate::internals::{EngineState, EntityId};

use super::accessing::Accessing;

/// A layer allowing for variable-width strings to exist within bricks by using
/// outgoing properties as extensions. We map a single string of N bytes into N / 256
/// smaller bricks of 256 bytes each and add them all as properties to a single object
/// with a specific identifier equal to the hash of the given string, and in so doing,
/// we intern it.
pub trait Strings {
    /// Hash a string into an entity identifier (basic hash string helper function)
    fn hash_string(str: &str) -> EntityId;
    /// Creates a string object and attached outgoing properties returning the same id
    /// if called multiple times, without creating new morphisms
    fn create_string_object(&self, str: &str) -> EntityId;
    /// Recovers the string value from the entity by joining together all of the string parts
    fn recover_string(&self, id: EntityId) -> Option<String>;
    /// Checks whether a string exists in the system
    fn string_exists(&self, str: &str) -> bool;
    /// Deletes the string from the system (will be reconstructed in the same way next time)
    fn delete_string(&self, str: &str);
}

impl Strings for Arc<EngineState> {
    fn hash_string(str: &str) -> EntityId {
        let mut hasher = DefaultHasher::new();
        str.hash(&mut hasher);
        hasher.finish().try_into().unwrap()
    }

    fn create_string_object(&self, str: &str) -> EntityId {
        fn split_str_into_parts(input: &str, part_size: usize) -> impl Iterator<Item = &str> {
            input
                .char_indices()
                .step_by(part_size)
                .map(move |(start, _)| &input[start..(start + part_size).min(input.len())])
        }

        let str_hash = Self::hash_string(str);

        if self.create_specific_object(str_hash).is_some() {
            for part in split_str_into_parts(str, 256) {
                let _ = self.add_outgoing_property_raw(
                    str_hash,
                    "String".into(),
                    part.as_bytes().to_vec(),
                );
            }
        }

        str_hash
    }

    fn recover_string(&self, id: EntityId) -> Option<String> {
        fn join_parts(parts: Vec<Vec<u8>>) -> Vec<u8> {
            let total_size: usize = parts.iter().map(|part| part.len()).sum();
            let mut joined: Vec<u8> = Vec::with_capacity(total_size);

            for part in parts {
                joined.extend(part);
            }

            joined
        }

        if !self.entity_exists(id) {
            return None;
        }

        let parts = self
            .query_access()
            .with_source(id)
            .with_component("String".into())
            .get()
            .as_slice()
            .iter()
            .flat_map(|&e| self.get_brick(e))
            .map(|e| e.data)
            .collect();

        Some(String::from_utf8_lossy(&join_parts(parts)).to_string())
    }

    fn string_exists(&self, str: &str) -> bool {
        let str_hash = Self::hash_string(str);
        self.entity_exists(str_hash)
    }

    fn delete_string(&self, str: &str) {
        let str_hash = Self::hash_string(str);
        self.destroy_object(str_hash);
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod strings_testing {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    use crate::internals::{EngineState, EntityId};

    use super::Strings;

    #[test]
    fn test_create_string() {
        let engine_state = EngineState::new();
        let hello_world = engine_state.create_string_object("hello world");

        let expected = {
            let mut hasher = DefaultHasher::new();
            "hello world".hash(&mut hasher);
            let str_hash: EntityId = hasher.finish().try_into().unwrap();
            str_hash
        };

        assert_eq!(expected, hello_world);
    }

    #[test]
    fn test_recover_string() {
        let engine_state = EngineState::new();
        let random_text = include_str!("random275.txt");
        let hello_world = engine_state.create_string_object(random_text);
        assert_eq!(
            Some(random_text.to_string()),
            engine_state.recover_string(hello_world)
        );
    }

    #[test]
    fn test_reuse_string() {
        let engine_state = EngineState::new();
        let random_text = include_str!("random275.txt");
        let hello_world1 = engine_state.create_string_object(random_text);
        let hello_world2 = engine_state.create_string_object(random_text);
        assert_eq!(hello_world1, hello_world2);
    }
}
