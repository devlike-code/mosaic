#[cfg(test)]
mod internals_tests {
    use crate::internals::{
        default_vals, load_mosaic_commands, self_val, Mosaic, MosaicCRUD, MosaicIO,
        MosaicTypelevelCRUD, TileType, Value,
    };

    #[test]
    fn test_commit() {
        let mosaic = Mosaic::new();
        mosaic.new_type("I: i32;").unwrap();
        mosaic.new_object("I", default_vals());

        if let Some(mut a) = mosaic.get_all().next() {
            assert_eq!(Value::I32(0), a["self"]);
            a["self"] = Value::I32(12);
            assert_eq!(Value::I32(12), a["self"]);
        }

        if let Some(a) = mosaic.get_all().next() {
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
        mosaic.new_type("A: void;").unwrap();
        mosaic.new_type("B: void;").unwrap();
        mosaic.new_type("A_to_B: void;").unwrap();
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

        let a = mosaic.new_object("DEBUG", default_vals());
        let b = mosaic.new_object("DEBUG", default_vals());
        let a_b = mosaic.new_arrow(&a, &b, "DEBUG", default_vals());

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b);
        assert!(!mosaic.is_tile_valid(&a_b_id));
    }

    #[test]
    fn test_component_field_indexing() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: product { x: i32, y: f32 };").unwrap();

        let mut a = mosaic.new_object("Foo", default_vals());
        assert_eq!(Value::I32(0), a["x"]);
        assert_eq!(Value::F32(0.0), a["y"]);

        a["x"] = Value::I32(7);
        assert_eq!(Value::I32(7), a["x"]);
    }

    #[test]
    fn test_alias_components_have_self_field() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let mut a = mosaic.new_object("Foo", default_vals());
        assert_eq!(Value::I32(0), a["self"]);

        a["self"] = Value::I32(7);
        assert_eq!(Value::I32(7), a["self"]);
    }

    fn test_data() -> [u8; 437] {
        [
            0, 12, 68, 69, 66, 85, 71, 58, 32, 118, 111, 105, 100, 59, 0, 48, 69, 114, 114, 111,
            114, 58, 32, 112, 114, 111, 100, 117, 99, 116, 32, 123, 32, 112, 111, 115, 105, 116,
            105, 111, 110, 58, 32, 115, 51, 50, 44, 32, 109, 101, 115, 115, 97, 103, 101, 58, 32,
            98, 49, 50, 56, 32, 125, 59, 0, 9, 70, 111, 111, 58, 32, 105, 51, 50, 59, 0, 11, 71,
            114, 111, 117, 112, 58, 32, 115, 51, 50, 59, 0, 16, 71, 114, 111, 117, 112, 79, 119,
            110, 101, 114, 58, 32, 115, 51, 50, 59, 0, 22, 80, 97, 114, 97, 109, 101, 116, 101,
            114, 66, 105, 110, 100, 105, 110, 103, 58, 32, 115, 51, 50, 59, 0, 13, 80, 114, 111,
            99, 101, 115, 115, 58, 32, 115, 51, 50, 59, 0, 22, 80, 114, 111, 99, 101, 115, 115, 80,
            97, 114, 97, 109, 101, 116, 101, 114, 58, 32, 115, 51, 50, 59, 0, 20, 80, 114, 111, 99,
            101, 115, 115, 82, 101, 115, 117, 108, 116, 58, 32, 118, 111, 105, 100, 59, 0, 20, 82,
            101, 115, 117, 108, 116, 66, 105, 110, 100, 105, 110, 103, 58, 32, 118, 111, 105, 100,
            59, 0, 13, 83, 116, 114, 105, 110, 103, 58, 32, 98, 49, 50, 56, 59, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 70,
            111, 111, 0, 0, 0, 4, 0, 0, 0, 101, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5, 68, 69, 66, 85, 71, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 5,
            68, 69, 66, 85, 71, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 5, 68, 69, 66, 85, 71, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 5, 68,
            69, 66, 85, 71, 0, 0, 0, 0,
        ]
    }

    #[test]
    fn test_save() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let a = mosaic.new_object("Foo", self_val(Value::I32(101)));
        let b = mosaic.new_object("DEBUG", default_vals());
        let c = mosaic.new_object("DEBUG", default_vals());
        let _ab = a.arrow_to(&b, "DEBUG", default_vals());
        //let _ab = mosaic.new_arrow(&a, &b, "DEBUG", default_vals());
        let _bc = b.arrow_to(&c, "DEBUG", default_vals());
        //let _bc = mosaic.new_arrow(&b, &c, "DEBUG", default_vals());

        assert_eq!(&test_data(), mosaic.save().as_slice());
    }

    #[test]
    fn test_save_sum_type() {
        let mosaic = Mosaic::new();

        let input = "Position : sum { x: i32, y: i32 z: i32};";
        mosaic.new_type(input).unwrap();

        mosaic.new_type("Foo: i32;").unwrap();

        let a = mosaic.new_object("Foo", self_val(Value::I32(101)));
        let mut s = mosaic.new_object(
            "Position",
            vec![
                ("x".into(), Value::I32(11)),
                ("x".into(), Value::F32(10.1)),
                ("y".into(), Value::I32(22)),
                ("z".into(), Value::I32(44)),
            ],
        );

        let b = mosaic.new_object("DEBUG", default_vals());
        let c = mosaic.new_object("DEBUG", default_vals());
        let _ab = mosaic.new_arrow(&a, &b, "DEBUG", default_vals());
        let _bc = mosaic.new_arrow(&b, &c, "DEBUG", default_vals());
        //println!("DEBUG TILE 'self' field: {:?}", b["self"]); //Alias doesn't have fields so indexer doesn't work

        println!("SUM TILE: {:?}", s);
        s["index"] = Value::S32("x".into());
        println!("SUM TILE: {:?}", s);

        // assert_eq!(&test_data(), mosaic.save().as_slice());
    }

    #[test]
    fn test_clean_load() {
        let data = test_data();
        let mosaic = Mosaic::new();

        let loaded = load_mosaic_commands(data.as_slice()).unwrap();
        assert_eq!(14, loaded.len());

        mosaic.load(data.as_slice()).unwrap();
        let new_obj = mosaic.new_object("DEBUG", default_vals());
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
        assert_eq!(14, loaded.len());

        let new_obj = mosaic.new_object("DEBUG", default_vals());
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
