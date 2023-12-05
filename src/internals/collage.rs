use itertools::Itertools;

use crate::internals::Tile;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Pick {
    Arrows,
    Descriptors,
    Extensions,
    Targets,
    Sources,
}

#[derive(Debug, Clone)]
pub enum Cut {
    Include(Vec<String>),
    Exclude(Vec<String>),
    Objects,
    Arrows,
    Descriptors,
    Extensions,
}

impl Cut {
    pub fn into_u8(&self) -> u8 {
        match self {
            Cut::Include(_) => 0,
            Cut::Exclude(_) => 1,
            Cut::Objects => 2,
            Cut::Arrows => 3,
            Cut::Descriptors => 4,
            Cut::Extensions => 5,
        }
    }
}

#[derive(Debug)]
pub enum Collage {
    Tiles,
    Gather(Vec<Box<Collage>>),
    Pick(Pick, Box<Collage>),
    Cut(Cut, Box<Collage>),
}

pub trait MosaicCollage {
    fn apply_collage(&self, mq: Box<Collage>, tiles: Option<Vec<Tile>>)
        -> std::vec::IntoIter<Tile>;
}

pub fn tiles() -> Box<Collage> {
    Box::new(Collage::Tiles)
}

pub fn arrows_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Pick(Pick::Arrows, mq))
}

pub fn descriptors_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Pick(Pick::Descriptors, mq))
}

pub fn extensions_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Pick(Pick::Extensions, mq))
}

pub fn targets_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Pick(Pick::Targets, mq))
}

pub fn sources_from(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Pick(Pick::Sources, mq))
}

pub fn take_components(comps: &[&str], mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Cut(
        Cut::Include(comps.iter().map(|s| s.to_string()).collect_vec()),
        mq,
    ))
}

pub fn leave_components(comps: &[&str], mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Cut(
        Cut::Exclude(comps.iter().map(|s| s.to_string()).collect_vec()),
        mq,
    ))
}

pub fn take_arrows(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Cut(Cut::Arrows, mq))
}

pub fn take_descriptors(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Cut(Cut::Descriptors, mq))
}

pub fn take_extensions(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Cut(Cut::Extensions, mq))
}

pub fn take_objects(mq: Box<Collage>) -> Box<Collage> {
    Box::new(Collage::Cut(Cut::Objects, mq))
}

pub fn gather(mqs: Vec<Box<Collage>>) -> Box<Collage> {
    Box::new(Collage::Gather(mqs))
}

#[cfg(test)]
mod query_utility_tests {
    use itertools::Itertools;

    use crate::internals::{targets_from, void, Mosaic, MosaicCRUD, MosaicIO};

    use super::{take_arrows, tiles, MosaicCollage};

    #[test]
    fn collage_test() {
        let mosaic = Mosaic::new();
        let t = mosaic.new_object("void", void());
        let u = mosaic.new_object("void", void());
        let v = mosaic.new_object("void", void());
        mosaic.new_arrow(&t, &u, "void", void());
        mosaic.new_arrow(&t, &v, "void", void());

        let mq = targets_from(take_arrows(tiles()));
        let mut result = mosaic.apply_collage(mq, None).collect_vec();

        result.sort();
        assert_eq!(vec![u.clone(), v.clone()], result);
    }

    #[test]
    fn collage_test_limited_to_some_tiles() {
        let mosaic = Mosaic::new();
        let t = mosaic.new_object("void", void());
        let u = mosaic.new_object("void", void());
        let v = mosaic.new_object("void", void());
        mosaic.new_arrow(&t, &u, "void", void());
        mosaic.new_arrow(&t, &v, "void", void());

        let mq = targets_from(take_arrows(tiles()));
        let selection = vec![t.clone(), u.clone()];
        let mut result = mosaic.apply_collage(mq, Some(selection)).collect_vec();

        result.sort();
        assert_eq!(vec![u.clone()], result);
    }
}
