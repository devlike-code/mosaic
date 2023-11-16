#[cfg(test)]
mod internals_tests {
    use crate::internals::{
        Mosaic, MosaicCRUD, MosaicGetEntities, MosaicTypelevelCRUD, TileCommit, TileType, Value,
    };

    #[test]
    fn test_commit() {
        let mosaic = Mosaic::new();
        mosaic.new_type("I: i32;").unwrap();
        {
            let mut a = mosaic.new_object("I");
            a["self"] = Value::I32(12);
            mosaic.commit(&a).unwrap();
        }

        if let Some(a) = mosaic.get_entities().next() {
            assert_eq!(Value::I32(12), a["self"]);
        }
    }

    #[test]
    fn test_basic_mosaic_usage() {
        let mosaic = Mosaic::new();
        mosaic.new_type("A: void;").unwrap();
        mosaic.new_type("B: void;").unwrap();
        mosaic.new_type("A_to_B: void;").unwrap();
        // We make two objects and an arrow: A --A_to_B--> B
        let a = mosaic.new_object("A");
        let b = mosaic.new_object("B");
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B");

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
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B");
        let a_b_id = a_b.id;
        mosaic.delete_tile(a);
        assert!(!mosaic.tile_exists(&a_id));
        assert!(!mosaic.tile_exists(&a_b_id));
    }

    #[test]
    fn test_cloning_isnt_affecting_mosaic() {
        let mosaic = Mosaic::new();
        mosaic.new_type("A: void;").unwrap();
        mosaic.new_type("B: void;").unwrap();
        mosaic.new_type("A_to_B: void;").unwrap();
        let a = mosaic.new_object("A");
        let b = mosaic.new_object("B");
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B");

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b.clone());
        assert!(!mosaic.tile_exists(&a_b_id));
    }

    #[test]
    fn test_cannot_commit_invalid_tile() {
        let mosaic = Mosaic::new();

        let a = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let a_b = mosaic.new_arrow(&a, &b, "DEBUG");

        let a_b_id = a_b.id;
        let cloned_a_b = a_b.clone();
        mosaic.delete_tile(a_b);
        assert!(!mosaic.tile_exists(&a_b_id));
        assert!(mosaic.commit(&cloned_a_b).is_err());
    }

    #[test]
    fn test_component_field_indexing() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: product { x: i32, y: f32 };").unwrap();

        let mut a = mosaic.new_object("Foo");
        assert_eq!(Value::I32(0), a["x"]);
        assert_eq!(Value::F32(0.0), a["y"]);

        a["x"] = Value::I32(7);
        assert_eq!(Value::I32(7), a["x"]);
    }

    #[test]
    fn test_alias_components_have_self_field() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let mut a = mosaic.new_object("Foo");
        assert_eq!(Value::I32(0), a["self"]);

        a["self"] = Value::I32(7);
        assert_eq!(Value::I32(7), a["self"]);
    }
}
