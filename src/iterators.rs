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

    use crate::internals::{Mosaic, MosaicCRUD, MosaicGetEntities};

    #[test]
    fn test_get_entities() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let _a_b = mosaic.new_arrow(&a, &b, "A -> B".into());
        // We want to select everything
        println!("{:?}", mosaic.get_entities().collect_vec());
    }

    #[test]
    fn test_get_tiles() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("A".into());
        let b = mosaic.new_object("B".into());
        let _a_b = mosaic.new_arrow(&a, &b, "A -> B".into());

        println!("{:?}", vec![a, b].into_iter().collect_vec());
    }
}
