#[cfg(test)]
mod test_iterators {
    use std::sync::Arc;

    use itertools::Itertools;

    use crate::{
        internals::{Mosaic, MosaicCRUD, MosaicGetEntities, MosaicTypelevelCRUD},
        iterators::{
            get_arrows_from::GetArrowsFromTiles, get_arrows_into::GetArrowsIntoTiles,
            get_dependents::GetDependentTiles, get_objects::GetObjects,
            get_sources::GetSourcesExtension, get_targets::GetTargets,
            include_component::IncludeComponent, include_components::IncludeComponents, get_descriptors::GetDescriptors,
        },
    };

    #[test]
    fn test_get_entities() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let _a_b = mosaic.new_arrow(&a, &b, "DEBUG");
        // We want to select everything
        let all_entities = mosaic.get_entities().collect_vec();
        assert_eq!(3, all_entities.len());
    }

    #[test]
    fn test_get_dependents() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let a_b = mosaic.new_arrow(&a, &b, "DEBUG");

        let mut dependents = a.iter_with(&mosaic).get_dependents();
        assert_eq!(dependents.next(), Some(a_b));
        assert_eq!(dependents.next(), None);
    }

    #[test]
    fn test_iterator_filters() {
        let mosaic = Mosaic::new();
        mosaic.new_type("C: void;").unwrap(); // An object in some Category
        mosaic.new_type("P: void;").unwrap(); // Property
        mosaic.new_type("C_to_C: void;").unwrap(); // C -> C
        mosaic.new_type("C_to_C_sqr: void;").unwrap(); // (C -> C) -> (C -> C)
        let a = mosaic.new_object("C");
        let b = mosaic.new_object("C");
        let c = mosaic.new_object("C");
        let a_p = mosaic.new_descriptor(&a, "P");
        let a_b = mosaic.new_arrow(&a, &b, "C_to_C");
        let a_c = mosaic.new_arrow(&a, &c, "C_to_C");
        let ab_ac = mosaic.new_arrow(&a_b, &a_c, "C_to_C_sqr");

        let a_arrows = a.get_arrows_with(&mosaic).collect_vec();
        assert_eq!(2, a_arrows.len());
        assert!(!a_arrows.contains(&a_p));
        assert!(a_arrows.contains(&a_b));
        assert!(a_arrows.contains(&a_c));

        let b_arrows = b.get_arrows_with(&mosaic).collect_vec();
        assert_eq!(1, b_arrows.len());
        assert!(b_arrows.contains(&a_b));

        let ab_arrows = a_b.get_arrows_with(&mosaic).collect_vec();

        assert_eq!(1, ab_arrows.len());
        assert!(ab_arrows.contains(&ab_ac));

        let zzz = a.iter_with(&mosaic).collect_vec();
        
        let a_desc_1 = a.iter_with(&mosaic).get_descriptors().collect_vec();
        let a_desc = a.get_descriptors_with(&mosaic).collect_vec();
        assert_eq!(1, a_desc.len());
        
        assert!(a_desc.contains(&a_p));

        let direct_arrows = mosaic
            .get_entities()
            .include_component("C_to_C")
            .collect_vec();
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
        mosaic.new_type("Src: void;").unwrap();
        mosaic.new_type("Tgt: void;").unwrap();
        mosaic.new_type("Arr: void;").unwrap();
        let src = mosaic.new_object("Src"); // 0
        let tgt1 = mosaic.new_object("Tgt"); // 1
        let tgt2 = mosaic.new_object("Tgt"); // 2
        let _a1 = mosaic.new_arrow(&src, &tgt1, "Arr"); // 3
        let _a2 = mosaic.new_arrow(&src, &tgt2, "Arr"); // 4

        let into_tgt1 = tgt1.iter_with(&mosaic).get_arrows_into().collect_vec();
        let into_tgt2 = tgt2.iter_with(&mosaic).get_arrows_into().collect_vec();
        assert_eq!(1, into_tgt1.len());
        assert_eq!(1, into_tgt2.len());
        assert_ne!(into_tgt1.first(), into_tgt2.first());
        let src1 = into_tgt1
            .into_iter()
            .get_sources_with(&Arc::clone(&mosaic))
            .next();

        let src2 = into_tgt2
            .into_iter()
            .get_sources_with(&Arc::clone(&mosaic))
            .next();

        assert_eq!(src1, src2);
    }

    #[test]
    fn test_filtering_by_arrow_type() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Src: void;").unwrap();
        mosaic.new_type("Tgt: void;").unwrap();
        mosaic.new_type("Arr1: void;").unwrap();
        mosaic.new_type("Arr2: void;").unwrap();
        mosaic.new_type("Arr3: void;").unwrap();
        let src = mosaic.new_object("Src");
        let src2 = mosaic.new_object("Src");
        let tgt1 = mosaic.new_object("Tgt");
        let tgt2 = mosaic.new_object("Tgt");
        let tgt3 = mosaic.new_object("Tgt");
        let _a1 = mosaic.new_arrow(&src, &tgt1, "Arr1");
        let _a2 = mosaic.new_arrow(&src, &tgt2, "Arr2");
        let _a3 = mosaic.new_arrow(&src, &tgt3, "Arr3");
        let _a4 = mosaic.new_arrow(&src2, &src, "Arr2");
        let mut p = mosaic
            .get_entities() // [ src, src2, tgt1, tgt2, tgt3, a1, a2, a3, a4 ]
            .get_objects() // [ src, src2, tgt1, tgt2, tgt3 ]
            .get_arrows_from() // treba: [ [ a1, a2, a3 ], [ a4 ], [], [], [] ],  mislim: [ a1, a2, a3, a4 ]
            .include_components(&["Arr2", "Arr3"]) // [ a2, a3, a4 ]
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
