#[cfg(test)]
mod test_iterators {
    use itertools::Itertools;

    use crate::{
        internals::{void, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD},
        iterators::{
            component_selectors::ComponentSelectors, tile_filters::TileFilters,
            tile_getters::TileGetters,
        },
    };

    #[test]
    fn test_get_entities() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", void());
        let b = mosaic.new_object("void", void());
        let _a_b = mosaic.new_arrow(&a, &b, "void", void());
        // We want to select everything
        let all_entities = mosaic.get_all().collect_vec();
        assert_eq!(3, all_entities.len());
    }

    #[test]
    fn test_get_dependents() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", void());
        let b = mosaic.new_object("void", void());
        let a_b = mosaic.new_arrow(&a, &b, "void", void());

        let mut dependents = a.into_iter().get_dependents();
        assert_eq!(dependents.next(), Some(a_b));
        assert_eq!(dependents.next(), None);
    }

    #[test]
    fn test_descriptor_directly_or_indirectly() {
        let mosaic = Mosaic::new();

        let a = mosaic.new_object("void", void());
        let a_p = mosaic.new_descriptor(&a, "void", void());
        let a_desc = a.clone().into_iter().get_descriptors().collect_vec();

        assert_eq!(Some(&a_p), a_desc.first());

        let a_desc2 = a
            .clone()
            .into_iter()
            .get_dependents()
            .filter_descriptors()
            .collect_vec();
        assert_eq!(Some(&a_p), a_desc2.first());

        let a_desc3 = a.into_iter().get_descriptors().collect_vec();
        assert_eq!(Some(&a_p), a_desc3.first());
    }

    #[test]
    fn test_iterator_filters() {
        let mosaic = Mosaic::new();
        mosaic.new_type("C: unit;").unwrap(); // An object in some Category
        mosaic.new_type("P: unit;").unwrap(); // Property
        mosaic.new_type("C_to_C: unit;").unwrap(); // C -> C
        mosaic.new_type("C_to_C_sqr: unit;").unwrap(); // (C -> C) -> (C -> C)
        let a = mosaic.new_object("C", void());
        let b = mosaic.new_object("C", void());
        let c = mosaic.new_object("C", void());
        let a_p = mosaic.new_descriptor(&a, "P", void());
        let a_b = mosaic.new_arrow(&a, &b, "C_to_C", void());
        let a_c = mosaic.new_arrow(&a, &c, "C_to_C", void());
        let ab_ac = mosaic.new_arrow(&a_b, &a_c, "C_to_C_sqr", void());

        let a_arrows = a.clone().into_iter().get_arrows().collect_vec();
        assert_eq!(2, a_arrows.len());
        assert!(!a_arrows.contains(&a_p));
        assert!(a_arrows.contains(&a_b));
        assert!(a_arrows.contains(&a_c));

        let b_arrows = b.into_iter().get_arrows().collect_vec();
        assert_eq!(1, b_arrows.len());
        assert!(b_arrows.contains(&a_b));

        let ab_arrows = a_b.clone().into_iter().get_arrows().collect_vec();

        assert_eq!(1, ab_arrows.len());
        assert!(ab_arrows.contains(&ab_ac));
        let a_desc = a.clone().into_iter().get_descriptors().collect_vec();
        assert_eq!(1, a_desc.len());

        assert!(a_desc.contains(&a_p));

        let direct_arrows = mosaic.get_all().include_component("C_to_C").collect_vec();
        assert_eq!(2, direct_arrows.len());
        assert!(direct_arrows.contains(&a_b));
        assert!(direct_arrows.contains(&a_c));
    }

    /*
           Src --a1---> Tgt1
            |
            |
            a2
            |
            v
           Tgt2
    */
    #[test]
    fn test_get_arrows_into() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Src: unit;").unwrap();
        mosaic.new_type("Tgt: unit;").unwrap();
        mosaic.new_type("Arr: unit;").unwrap();
        let src = mosaic.new_object("Src", void()); // 0
        let tgt1 = mosaic.new_object("Tgt", void()); // 1
        let tgt2 = mosaic.new_object("Tgt", void()); // 2
        let _a1 = mosaic.new_arrow(&src, &tgt1, "Arr", void()); // 3
        let _a2 = mosaic.new_arrow(&src, &tgt2, "Arr", void()); // 4

        let into_tgt1 = tgt1.into_iter().get_arrows_into().collect_vec();
        let into_tgt2 = tgt2.into_iter().get_arrows_into().collect_vec();
        assert_eq!(1, into_tgt1.len());
        assert_eq!(1, into_tgt2.len());
        assert_ne!(into_tgt1.first(), into_tgt2.first());
        let src1 = into_tgt1.into_iter().get_sources().next();

        let src2 = into_tgt2.into_iter().get_sources().next();

        assert_eq!(src1, src2);
    }

    #[test]
    fn test_filtering_by_arrow_type() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Src: unit;").unwrap();
        mosaic.new_type("Tgt: unit;").unwrap();
        mosaic.new_type("Arr1: unit;").unwrap();
        mosaic.new_type("Arr2: unit;").unwrap();
        mosaic.new_type("Arr3: unit;").unwrap();
        let src = mosaic.new_object("Src", void());
        let src2 = mosaic.new_object("Src", void());
        let tgt1 = mosaic.new_object("Tgt", void());
        let tgt2 = mosaic.new_object("Tgt", void());
        let tgt3 = mosaic.new_object("Tgt", void());
        let _a1 = mosaic.new_arrow(&src, &tgt1, "Arr1", void());
        let _a2 = mosaic.new_arrow(&src, &tgt2, "Arr2", void());
        let _a3 = mosaic.new_arrow(&src, &tgt3, "Arr3", void());
        let _a4 = mosaic.new_arrow(&src2, &src, "Arr2", void());

        let mut p = mosaic
            .get_all() // [ src, src2, tgt1, tgt2, tgt3, a1, a2, a3, a4 ]
            .filter_objects() // [ src, src2, tgt1, tgt2, tgt3 ]
            .get_arrows_from() // treba: [ [ a1, a2, a3 ], [ a4 ], [], [], [] ],  mislim: [ a1, a2, a3, a4 ]
            .include_components(&["Arr2".to_string(), "Arr3".to_string()]) // [ a2, a3, a4 ]
            .get_targets() // [ tgt2, tgt3, src ]
            .collect_vec();
        p.sort();
        let mut p = p.into_iter();
        assert_eq!(Some(src), p.next());
        assert_eq!(Some(tgt2), p.next());
        assert_eq!(Some(tgt3), p.next());
        assert_eq!(None, p.next());
    }
}
