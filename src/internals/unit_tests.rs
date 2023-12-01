#[cfg(test)]
mod internals_tests {
    use crate::internals::tile_access::{TileFieldQuery, TileFieldSetter};
    use crate::internals::{
        default_vals, load_mosaic_commands, self_val, Mosaic, MosaicCRUD, MosaicIO,
        MosaicTypelevelCRUD, TileType, Value,
    };

    #[test]
    fn test_reading_value_after_dropping() {
        let mosaic = Mosaic::new();
        mosaic.new_type("I: i32;").unwrap();
        mosaic.new_object("I", default_vals());

        if let Some(mut a) = mosaic.get_all().next() {
            assert_eq!(Value::I32(0), a.get("self"));
            a.set("self", 12i32);

            assert_eq!(Value::I32(12), a.get("self"));
        }

        if let Some(a) = mosaic.get_all().next() {
            assert_eq!(Value::I32(12), a.get("self"));
        }
    }

    #[test]
    fn test_tuple_get() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Position: { x: i32, y: i32 };").unwrap();
        let mut a = mosaic.new_object("Position", default_vals());

        a.set("x", 35i32);
        a.set("y", 64i32);

        if let (Value::I32(x), Value::I32(y)) = a.get_by(("x", "y")) {
            assert_eq!(35, x);
            assert_eq!(64, y);
        } else {
            unreachable!();
        }
    }

    #[test]
    fn test_basic_mosaic_usage() {
        let mosaic = Mosaic::new();
        mosaic.new_type("A: unit;").unwrap();
        mosaic.new_type("B: unit;").unwrap();
        mosaic.new_type("A_to_B: unit;").unwrap();
        // We make two objects and an arrow: A --A_to_B--> B
        let a = mosaic.new_object("A", default_vals());
        let b = mosaic.new_object("B", default_vals());
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B", default_vals());

        // Check whether everything exists
        assert!(mosaic.is_tile_valid(&a));
        assert!(mosaic.is_tile_valid(&b));
        assert!(mosaic.is_tile_valid(&a_b));
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
        assert!(!mosaic.is_tile_valid(&a_b_id));

        // Create new arrow with the same endpoints, and then
        // delete one of those endpoints; we're expecting the arrows
        // to disappear as well
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B", default_vals());
        let a_b_id = a_b.id;
        mosaic.delete_tile(a);
        assert!(!mosaic.is_tile_valid(&a_id));
        assert!(!mosaic.is_tile_valid(&a_b_id));
    }

    #[test]
    fn test_cloning_isnt_affecting_mosaic() {
        let mosaic = Mosaic::new();
        mosaic.new_type("A: unit;").unwrap();
        mosaic.new_type("B: unit;").unwrap();
        mosaic.new_type("A_to_B: unit;").unwrap();
        let a = mosaic.new_object("A", default_vals());
        let b = mosaic.new_object("B", default_vals());
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B", default_vals());

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b.clone());
        assert!(!mosaic.is_tile_valid(&a_b_id));
    }

    #[test]
    fn test_cannot_commit_invalid_tile() {
        let mosaic = Mosaic::new();

        let a = mosaic.new_object("void", default_vals());
        let b = mosaic.new_object("void", default_vals());
        let a_b = mosaic.new_arrow(&a, &b, "void", default_vals());

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b);
        assert!(!mosaic.is_tile_valid(&a_b_id));
    }

    #[test]
    fn test_component_field_indexing() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: { x: i32, y: f32 };").unwrap();

        let mut a = mosaic.new_object("Foo", default_vals());
        assert_eq!(0, a.get("x").as_i32());
        assert_eq!(0.0, a.get("y").as_f32());

        a.set("x", 7i32);
        assert_eq!(7, a.get("x").as_i32());
    }

    #[test]
    fn test_alias_components_have_self_field() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let mut a = mosaic.new_object("Foo", default_vals());
        assert_eq!(0, a.get("self").as_i32());

        a.set("self", 7i32);
        assert_eq!(7, a.get("self").as_i32());
    }

    fn test_data() -> [u8; 424] {
        [
            0, 40, 69, 114, 114, 111, 114, 58, 32, 123, 32, 112, 111, 115, 105, 116, 105, 111, 110,
            58, 32, 115, 51, 50, 44, 32, 109, 101, 115, 115, 97, 103, 101, 58, 32, 115, 49, 50, 56,
            32, 125, 59, 0, 9, 70, 111, 111, 58, 32, 105, 51, 50, 59, 0, 11, 71, 114, 111, 117,
            112, 58, 32, 115, 51, 50, 59, 0, 16, 71, 114, 111, 117, 112, 79, 119, 110, 101, 114,
            58, 32, 115, 51, 50, 59, 0, 22, 80, 97, 114, 97, 109, 101, 116, 101, 114, 66, 105, 110,
            100, 105, 110, 103, 58, 32, 115, 51, 50, 59, 0, 13, 80, 114, 111, 99, 101, 115, 115,
            58, 32, 115, 51, 50, 59, 0, 22, 80, 114, 111, 99, 101, 115, 115, 80, 97, 114, 97, 109,
            101, 116, 101, 114, 58, 32, 115, 51, 50, 59, 0, 20, 80, 114, 111, 99, 101, 115, 115,
            82, 101, 115, 117, 108, 116, 58, 32, 117, 110, 105, 116, 59, 0, 20, 82, 101, 115, 117,
            108, 116, 66, 105, 110, 100, 105, 110, 103, 58, 32, 117, 110, 105, 116, 59, 0, 13, 83,
            116, 114, 105, 110, 103, 58, 32, 115, 49, 50, 56, 59, 0, 11, 118, 111, 105, 100, 58,
            32, 117, 110, 105, 116, 59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 70, 111, 111, 0, 0, 0, 4, 0, 0, 0, 101, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0,
            4, 118, 111, 105, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0,
            0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 118, 111, 105, 100, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 4,
            118, 111, 105, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0,
            0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 118, 111, 105, 100, 0, 0, 0, 0,
        ]
    }

    #[test]
    fn test_save() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let a = mosaic.new_object("Foo", self_val(Value::I32(101)));
        let b = mosaic.new_object("void", default_vals());
        let c = mosaic.new_object("void", default_vals());
        let _ab = a.arrow_to(&b, "void", default_vals());
        let _bc = b.arrow_to(&c, "void", default_vals());
        println!("{:?}", mosaic.save().as_slice());
        assert_eq!(&test_data(), mosaic.save().as_slice());
    }

    #[test]
    fn test_clean_load() {
        let data = test_data();
        let mosaic = Mosaic::new();

        let loaded = load_mosaic_commands(data.as_slice()).unwrap();
        assert_eq!(16, loaded.len());

        mosaic.load(data.as_slice()).unwrap();
        let new_obj = mosaic.new_object("void", default_vals());
        assert!(mosaic.is_tile_valid(&0));
        assert!(mosaic.is_tile_valid(&1));
        assert!(mosaic.is_tile_valid(&2));
        assert!(mosaic.is_tile_valid(&3));
        assert!(mosaic.is_tile_valid(&4));
        assert!(mosaic.is_tile_valid(&new_obj));
        assert_eq!(5, new_obj.id);
    }

    #[test]
    fn test_transitioning_load() {
        let data = test_data();
        let mosaic = Mosaic::new();

        let loaded = load_mosaic_commands(data.as_slice()).unwrap();
        assert_eq!(16, loaded.len());

        let new_obj = mosaic.new_object("void", default_vals());
        mosaic.load(data.as_slice()).unwrap();

        assert!(mosaic.is_tile_valid(&1));
        assert!(mosaic.is_tile_valid(&2));
        assert!(mosaic.is_tile_valid(&3));
        assert!(mosaic.is_tile_valid(&4));
        assert!(mosaic.is_tile_valid(&5));
        assert!(mosaic.is_tile_valid(&new_obj));
        assert_eq!(0, new_obj.id);
    }
}
