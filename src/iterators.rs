pub mod filter_with_component;
pub mod get_arrows;
pub mod get_arrows_from;
pub mod get_arrows_into;
pub mod get_dependents;
pub mod get_descriptors;
pub mod get_extensions;
pub mod get_loops;
pub mod get_objects;
pub mod just_tile;

#[cfg(test)]
mod test_iterators {
    use itertools::Itertools;

    use crate::{
        internals::{Mosaic, MosaicCRUD, MosaicGetEntities},
        iterators::{
            filter_with_component::FilterWithComponent, get_dependents::GetDependentTiles,
        },
    };

    #[test]
    fn test_get_entities() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let _a_b = mosaic.new_arrow(&a, &b, "A -> B".into());
        // We want to select everything
        let all_entities = mosaic.get_entities().collect_vec();
        assert_eq!(3, all_entities.len());
    }

    #[test]
    fn test_get_dependents() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let a_b = mosaic.new_arrow(&a, &b, "A -> B".into());

        let mut dependents = a.iter_with(&mosaic).get_dependents();
        assert_eq!(dependents.next(), Some(a_b));
        assert_eq!(dependents.next(), None);
    }

    #[test]
    fn test_iterator_filters() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("C".into());
        let b = mosaic.new_object("C".into());
        let c = mosaic.new_object("C".into());
        let a_p = mosaic.new_descriptor(&a, "P".into());
        let a_b = mosaic.new_arrow(&a, &b, "C -> C".into());
        let a_c = mosaic.new_arrow(&a, &c, "C -> C".into());
        let ab_ac = mosaic.new_arrow(&a_b, &a_c, "(C -> C) -> (C -> C)".into());

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
            .filter_component("C -> C")
            .collect_vec();
        assert_eq!(2, direct_arrows.len());
        assert!(direct_arrows.contains(&a_b));
        assert!(direct_arrows.contains(&a_c));
    }
}
