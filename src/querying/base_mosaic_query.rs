use itertools::Itertools;

use crate::internals::Tile;

pub enum Composite {
    Arrows,
    Descriptors,
    Extensions,
    Targets,
    Sources,
}

pub enum Cut {
    Include(Vec<String>),
    Exclude(Vec<String>),
    Objects,
    Arrows,
    Descriptors,
    Extensions,
}

pub enum Collage {
    Tiles,
    Gather(Composite, Box<Collage>),
    Filter(Cut, Box<Collage>),
}

pub trait MosaicCollage {
    fn apply_collage(
        &self,
        mq: Box<super::base_mosaic_query::Collage>,
        tiles: Option<Vec<Tile>>,
    ) -> std::vec::IntoIter<crate::internals::Tile>;
}

pub fn tiles() -> Box<Collage> {
    Box::new(Collage::Tiles)
}

pub fn arrows_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Gather(Composite::Arrows, mq))
}

pub fn descriptors_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Gather(Composite::Descriptors, mq))
}

pub fn extensions_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Gather(Composite::Extensions, mq))
}

pub fn targets_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Gather(Composite::Targets, mq))
}

pub fn sources_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Gather(Composite::Sources, mq))
}

pub fn take_components(comps: &[&str], mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Filter(
        Cut::Include(comps.iter().map(|s| s.to_string()).collect_vec()),
        mq,
    ))
}

pub fn leave_components(comps: &[&str], mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Filter(
        Cut::Exclude(comps.iter().map(|s| s.to_string()).collect_vec()),
        mq,
    ))
}

pub fn take_arrows(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Filter(Cut::Arrows, mq))
}

pub fn take_descriptors(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Filter(Cut::Descriptors, mq))
}

pub fn take_extensions(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Filter(Cut::Extensions, mq))
}

pub fn take_objects(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Filter(Cut::Objects, mq))
}

#[cfg(test)]
mod query_utility_tests {
    use itertools::Itertools;

    use crate::{
        internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO},
        querying::base_mosaic_query::targets_from,
    };

    use super::{take_arrows, tiles, MosaicCollage};

    #[test]
    fn collage_test() {
        let mosaic = Mosaic::new();
        let t = mosaic.new_object("void", default_vals());
        let u = mosaic.new_object("void", default_vals());
        let v = mosaic.new_object("void", default_vals());
        mosaic.new_arrow(&t, &u, "void", default_vals());
        mosaic.new_arrow(&t, &v, "void", default_vals());

        let mq = targets_from(take_arrows(tiles()));
        let mut result = mosaic.apply_collage(mq, None).collect_vec();

        result.sort();
        assert_eq!(vec![u.clone(), v.clone()], result);
    }

    #[test]
    fn collage_test_limited_to_some_tiles() {
        let mosaic = Mosaic::new();
        let t = mosaic.new_object("void", default_vals());
        let u = mosaic.new_object("void", default_vals());
        let v = mosaic.new_object("void", default_vals());
        mosaic.new_arrow(&t, &u, "void", default_vals());
        mosaic.new_arrow(&t, &v, "void", default_vals());

        let mq = targets_from(take_arrows(tiles()));
        let selection = vec![t.clone(), u.clone()];
        let mut result = mosaic.apply_collage(mq, Some(selection)).collect_vec();

        result.sort();
        assert_eq!(vec![u.clone()], result);
    }
}
