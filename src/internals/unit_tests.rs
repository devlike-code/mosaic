#[cfg(test)]
mod internals_tests {
    use random_string::generate;

    use crate::internals::tile_access::TileFieldSetter;
    use crate::internals::{
        load_mosaic_commands, par, pars, void, ComponentValuesBuilderSetter, Mosaic, MosaicCRUD,
        MosaicIO, MosaicTypelevelCRUD, TileType, Value,
    };

    #[test]
    fn test_reading_value_after_dropping() {
        let mosaic = Mosaic::new();
        mosaic.new_type("I: i32;").unwrap();
        mosaic.new_object("I", void());

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
    fn test_basic_mosaic_usage() {
        let mosaic = Mosaic::new();
        mosaic.new_type("A: unit;").unwrap();
        mosaic.new_type("B: unit;").unwrap();
        mosaic.new_type("A_to_B: unit;").unwrap();
        // We make two objects and an arrow: A --A_to_B--> B
        let a = mosaic.new_object("A", void());
        let b = mosaic.new_object("B", void());
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B", void());

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
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B", void());
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
        let a = mosaic.new_object("A", void());
        let b = mosaic.new_object("B", void());
        let a_b = mosaic.new_arrow(&a, &b, "A_to_B", void());

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b.clone());
        assert!(!mosaic.is_tile_valid(&a_b_id));
    }

    #[test]
    fn test_cannot_commit_invalid_tile() {
        let mosaic = Mosaic::new();

        let a = mosaic.new_object("void", void());
        let b = mosaic.new_object("void", void());
        let a_b = mosaic.new_arrow(&a, &b, "void", void());

        let a_b_id = a_b.id;
        mosaic.delete_tile(a_b);
        assert!(!mosaic.is_tile_valid(&a_b_id));
    }

    #[test]
    fn test_component_field_indexing() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: { x: i32, y: f32 };").unwrap();

        let mut a = mosaic.new_object("Foo", void());
        assert_eq!(0, a.get("x").as_i32());
        assert_eq!(0.0, a.get("y").as_f32());

        a.set("x", 7i32);
        assert_eq!(7, a.get("x").as_i32());
    }

    #[test]
    fn test_alias_components_have_self_field() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let mut a = mosaic.new_object("Foo", void());
        assert_eq!(0, a.get("self").as_i32());

        a.set("self", 7i32);
        assert_eq!(7, a.get("self").as_i32());
    }

    #[test]
    fn test_aliasing_types() {
        let mosaic = Mosaic::new();
        assert!(mosaic.component_registry.has_component_type(&"void".into()));
        mosaic.new_type("void2: void;").unwrap();
        assert!(mosaic
            .component_registry
            .has_component_type(&"void2".into()));
    }

    fn test_data() -> [u8; 229] {
        [
            0, 9, 70, 111, 111, 58, 32, 105, 51, 50, 59, 0, 11, 118, 111, 105, 100, 58, 32, 117,
            110, 105, 116, 59, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 70, 111, 111, 0, 0, 0, 4, 0, 0, 0, 101, 0, 0, 0, 0, 0,
            0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 4, 118,
            111, 105, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
            0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 118, 111, 105, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 4, 118, 111,
            105, 100, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0,
            0, 2, 0, 0, 0, 0, 0, 0, 0, 4, 118, 111, 105, 100, 0, 0, 0, 0,
        ]
    }

    #[test]
    fn test_save() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Foo: i32;").unwrap();

        let a = mosaic.new_object("Foo", par(101i32));
        let b = mosaic.new_object("void", void());
        let c = mosaic.new_object("void", void());
        let _ab = a.arrow_to(&b, "void", void());
        let _bc = b.arrow_to(&c, "void", void());
        println!("{:?}", mosaic.save().as_slice());
        assert_eq!(&test_data(), mosaic.save().as_slice());
    }

    #[test]
    fn test_clean_load() {
        let mosaic = Mosaic::new();

        let data = test_data();

        let loaded = load_mosaic_commands(data.as_slice()).unwrap();
        assert_eq!(7, loaded.len());

        mosaic.load(data.as_slice()).unwrap();
        let new_obj = mosaic.new_object("void", void());
        assert!(mosaic.is_tile_valid(&0));
        assert!(mosaic.is_tile_valid(&1));
        assert!(mosaic.is_tile_valid(&2));
        assert!(mosaic.is_tile_valid(&3));
        assert!(mosaic.is_tile_valid(&4));
        assert!(mosaic.is_tile_valid(&new_obj));
        assert_eq!(5, new_obj.id);
    }

    #[test]
    fn test_strings() {
        let mosaic = Mosaic::new();
        mosaic.new_type("S: str;").unwrap();
        let o = mosaic.new_object("S", par("hello world".to_string()));
        assert_eq!("hello world".to_string(), o.get("self").as_str());
    }

    #[test]
    fn test_really_big_strings() {
        let mosaic = Mosaic::new();
        let charset = "1234567890abcdefghijklmnopqrstuvwxyz.,!?";
        let size = 256 * 256 * 256;
        let str = generate(size, charset);
        assert_eq!(size, str.len());
        mosaic.new_type("S: str;").unwrap();
        let o = mosaic.new_object("S", par(str.clone()));
        assert_eq!(str, o.get("self").as_str());
    }

    #[test]
    fn test_really_big_strings_in_structs() {
        let mosaic = Mosaic::new();
        let charset = "1234567890abcdefghijklmnopqrstuvwxyz.,!?";
        let size = 256 * 256 * 256;
        let str1 = generate(size + 5, charset);
        let str2 = generate(size + 6, charset);
        assert_eq!(size + 5, str1.len());
        assert_eq!(size + 6, str2.len());
        mosaic.new_type("S2: { a: str, i: u32, b: str };").unwrap();
        let o = mosaic.new_object(
            "S2",
            pars()
                .set("a", str1.clone())
                .set("b", str2.clone())
                .set("i", 8u32)
                .ok(),
        );
        assert_eq!(str2, o.get("b").as_str());
        assert_eq!(8, o.get("i").as_u32());
        assert_eq!(str1, o.get("a").as_str());
    }

    #[test]
    fn test_transitioning_load() {
        let data = test_data();
        let mosaic = Mosaic::new();

        let loaded = load_mosaic_commands(data.as_slice()).unwrap();
        assert_eq!(7, loaded.len());

        let new_obj = mosaic.new_object("void", void());
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
