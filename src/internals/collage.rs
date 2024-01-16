use itertools::Itertools;

use crate::internals::Tile;

use super::EntityId;

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
    Tiles(Option<Vec<EntityId>>),
    Gather(Vec<Box<Collage>>),
    Pick(Pick, Box<Collage>),
    Cut(Cut, Box<Collage>),
}

pub trait MosaicCollage {
    fn apply_collage(&self, mq: &Collage, tiles: Option<Vec<Tile>>) -> std::vec::IntoIter<Tile>;
}

pub fn all_tiles() -> Box<Collage> {
    Box::new(Collage::Tiles(None))
}

pub fn tiles(tiles: Vec<Tile>) -> Box<Collage> {
    Box::new(Collage::Tiles(Some(
        tiles.iter().map(|t| t.id).collect_vec(),
    )))
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
