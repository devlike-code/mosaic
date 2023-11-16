#![allow(dead_code)]

pub mod byte_utilities;
pub mod component_grammar;
pub mod datatypes;
pub mod either;
pub mod entity_registry;
pub mod get_entities;
pub mod get_tiles;
pub mod logging;
pub mod mosaic;
pub mod sparse_matrix;
pub mod sparse_set;
pub mod tile;

pub use byte_utilities::*;
pub use datatypes::*;
pub use entity_registry::*;
pub use logging::*;
pub use mosaic::*;
pub use sparse_set::*;
pub use tile::*;

#[cfg(test)]
mod internals_tests {
    use crate::internals::{Mosaic, TileCommit, TileType};

    use super::MosaicCRUD;

    #[test]
    fn test_basic_mosaic_usage() {
        let mosaic = Mosaic::new();

        // We make two objects and an arrow: A --A_B--> B
        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let a_b = mosaic.new_arrow(&a, &b, "A -> B".into());

        // Check whether everything exists
        assert!(mosaic.tile_exists(&a));
        assert!(mosaic.tile_exists(&b));
        assert!(mosaic.tile_exists(&a_b));
        assert!(a.is_object());
        assert!(b.is_object());
        assert!(a_b.is_arrow());

        // Check whether the tile can be deconstructed
        if let TileType::Arrow { source, target } = a_b.tile_type {
            assert_eq!(a.id, source);
            assert_eq!(b.id, target);
        }

        // Let's cache the IDs to check them after deletion
        let a_id = a.id;
        let a_b_id = a_b.id;

        // Delete and check that this ID no longer exists
        mosaic.delete_tile(a_b);
        assert!(!mosaic.tile_exists(&a_b_id));

        // Create new arrow with the same endpoints, and then
        // delete one of those endpoints; we're expecting the arrows
        // to disappear as well
        let a_b = mosaic.new_arrow(&a, &b, "A -> B".into());
        let a_b_id = a_b.id;
        mosaic.delete_tile(a);
        assert!(!mosaic.tile_exists(&a_id));
        assert!(!mosaic.tile_exists(&a_b_id));
    }

    #[test]
    fn test_cloning_isnt_affecting_mosaic() {
        let mosaic = Mosaic::new();

        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let a_b = mosaic.new_arrow(&a, &b, "A -> B".into());

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b.clone());
        assert!(!mosaic.tile_exists(&a_b_id));
    }

    #[test]
    fn test_cannot_commit_invalid_tile() {
        let mosaic = Mosaic::new();

        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let a_b = mosaic.new_arrow(&a, &b, "A -> B".into());

        let a_b_id = a_b.id;
        let cloned_a_b = a_b.clone();
        mosaic.delete_tile(a_b);
        assert!(!mosaic.tile_exists(&a_b_id));
        assert!(mosaic.commit(&cloned_a_b).is_err());
    }
}
