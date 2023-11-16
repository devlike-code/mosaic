#[cfg(test)]
mod test_iterators {
    use itertools::Itertools;

    use crate::{
        internals::{Mosaic, MosaicCRUD, MosaicGetEntities, MosaicTypelevelCRUD},
        iterators::{
            filter_with_component::FilterWithComponent, get_dependents::GetDependentTiles,
        },
    };

    #[test]
    fn test_get_entities() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG".into());
        let b = mosaic.new_object("DEBUG".into());
        let _a_b = mosaic.new_arrow(&a, &b, "DEBUG".into());
        // We want to select everything
        let all_entities = mosaic.get_entities().collect_vec();
        assert_eq!(3, all_entities.len());
    }

    #[test]
    fn test_get_dependents() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG".into());
        let b = mosaic.new_object("DEBUG".into());
        let a_b = mosaic.new_arrow(&a, &b, "DEBUG".into());

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
        let a = mosaic.new_object("C".into());
        let b = mosaic.new_object("C".into());
        let c = mosaic.new_object("C".into());
        let a_p = mosaic.new_descriptor(&a, "P".into());
        let a_b = mosaic.new_arrow(&a, &b, "C_to_C".into());
        let a_c = mosaic.new_arrow(&a, &c, "C_to_C".into());
        let ab_ac = mosaic.new_arrow(&a_b, &a_c, "C_to_C_sqr".into());

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

        let a_desc = a.get_descriptors_with(&mosaic).collect_vec();
        assert_eq!(1, a_desc.len());
        assert!(a_desc.contains(&a_p));

        let direct_arrows = mosaic
            .get_entities()
            .filter_component("C_to_C")
            .collect_vec();
        assert_eq!(2, direct_arrows.len());
        assert!(direct_arrows.contains(&a_b));
        assert!(direct_arrows.contains(&a_c));
    }
}
