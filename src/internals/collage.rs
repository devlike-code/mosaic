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

#[cfg(test)]
mod query_utility_tests {
    use std::collections::HashSet;

    use itertools::Itertools;

    use crate::{
        capabilities::{CollageExportCapability, CollageImportCapability, SelectionCapability},
        internals::{
            all_tiles, descriptors_from, par, targets_from, void, Mosaic, MosaicCRUD, MosaicIO,
            MosaicTypelevelCRUD, Tile,
        },
    };

    use super::{arrows_from, take_arrows, take_components, tiles, MosaicCollage};

    #[test]
    fn collage_test() {
        let mosaic = Mosaic::new();
        let t = mosaic.new_object("void", void());
        let u = mosaic.new_object("void", void());
        let v = mosaic.new_object("void", void());
        mosaic.new_arrow(&t, &u, "void", void());
        mosaic.new_arrow(&t, &v, "void", void());

        let mq = targets_from(take_arrows(all_tiles()));
        let mut result = mosaic.apply_collage(&mq, None).collect_vec();

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

        let mq = targets_from(take_arrows(all_tiles()));
        let selection = vec![t.clone(), u.clone()];
        let mut result = mosaic.apply_collage(&mq, Some(selection)).collect_vec();

        result.sort();
        assert_eq!(vec![u.clone()], result);
    }

    #[test]
    fn test_getting_selection() {
        let mosaic = Mosaic::new();
        let t = mosaic.new_object("void", void());
        let u = mosaic.new_object("void", void());
        let v = mosaic.new_object("void", void());
        mosaic.new_arrow(&t, &u, "void", void());
        mosaic.new_arrow(&t, &v, "void", void());

        let s = mosaic.make_selection(&[t.clone(), u.clone(), v.clone()]);
        let expected: HashSet<Tile> = HashSet::from_iter(vec![t.clone(), u.clone(), v.clone()]);
        assert_eq!(expected, HashSet::from_iter(mosaic.get_selection(&s)),);

        let w = mosaic.new_object("void", void());
        let s = mosaic.make_selection(&[t.clone(), u.clone(), v.clone(), w.clone()]);
        let expected: HashSet<Tile> =
            HashSet::from_iter(vec![t.clone(), u.clone(), v.clone(), w.clone()]);
        assert_eq!(expected, HashSet::from_iter(mosaic.get_selection(&s)));
    }

    #[test]
    fn collage_from_tile_apply() {
        let mosaic = Mosaic::new();
        let _ = mosaic.new_type("Label : s32;");
        let t = mosaic.new_object("void", void());
        let u = mosaic.new_object("Label", par("test"));
        let v = mosaic.new_object("void", void());
        let _x = mosaic.new_arrow(&t, &u, "void", void());
        let _y = mosaic.new_arrow(&t, &v, "void", void());

        let collage = take_components(&["Label"], targets_from(arrows_from(tiles(vec![t]))));
        let mut selection = mosaic.apply_collage(&collage, None).unique().collect_vec();
        selection.sort();
        assert_eq!(vec![u.clone()], selection);

        let tile = collage.to_tiles(&mosaic);
        let collage_from_tile = tile.to_collage().unwrap();
        let mut selection_second = mosaic
            .apply_collage(&collage_from_tile, None)
            .unique()
            .collect_vec();
        selection_second.sort();

        assert_eq!(vec![u.clone()], selection_second);
    }
}
