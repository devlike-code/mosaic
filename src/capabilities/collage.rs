/*

pub enum Pick {
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
    Gather(Vec<Box<Collage>>),
    Pick(Pick, Box<Collage>),
    Cut(Cut, Box<Collage>),
}

*/

use std::sync::Arc;

use itertools::Itertools;

use crate::{
    internals::{
        arrows_from, descriptors_from, extensions_from, gather, leave_components, par,
        sources_from, take_arrows, take_components, take_descriptors, take_extensions,
        take_objects, targets_from, tiles, void, Collage, Cut, Mosaic, MosaicCRUD, MosaicIO,
        MosaicTypelevelCRUD, Pick, Tile,
    },
    iterators::{component_selectors::ComponentSelectors, tile_getters::TileGetters},
};

use super::StringCapability;

pub trait CollageCapability {
    fn make_collage(&self) -> Tile;
    fn apply_collage_pick(&self, pick: Pick, target: &Tile) -> Tile;
    fn apply_collage_gather(&self, subs: &[Tile]) -> Tile;
    fn apply_collage_cut(&self, cut: Cut, target: &Tile) -> Tile;
}

impl CollageCapability for Arc<Mosaic> {
    fn make_collage(&self) -> Tile {
        self.new_type("Collage: unit;").unwrap();
        self.new_type("CollagePick: u8;").unwrap();
        self.new_type("CollageCut: u8;").unwrap();
        self.new_type("CollageGather: unit;").unwrap();
        self.new_object("Collage", void())
    }

    fn apply_collage_pick(&self, pick: Pick, target: &Tile) -> Tile {
        self.new_extension(target, "CollagePick", par(pick as u8))
    }

    fn apply_collage_gather(&self, subs: &[Tile]) -> Tile {
        let gather = self.new_object("CollageGather", void());
        subs.iter().for_each(|sub| {
            self.new_arrow(&gather, sub, "CollageGather", void());
        });

        gather
    }

    fn apply_collage_cut(&self, cut: Cut, target: &Tile) -> Tile {
        let en = cut.into_u8();
        let cut_tile = self.new_extension(target, "CollageCut", par(en));
        let strings = match cut {
            Cut::Include(comps) => comps,
            Cut::Exclude(comps) => comps,
            _ => vec![],
        };

        strings.clone().iter().for_each(|s| {
            let tile = self.create_string_object(s).unwrap();
            self.new_arrow(&cut_tile, &tile, "CollageCut", par(en));
        });

        cut_tile
    }
}

trait CollageExportCapability {
    fn to_tiles(&self, mosaic: &Arc<Mosaic>) -> Tile;
}

trait CollageImportCapability {
    fn to_collage(&self) -> Option<Box<Collage>>;
}

impl CollageExportCapability for Box<Collage> {
    fn to_tiles(&self, mosaic: &Arc<Mosaic>) -> Tile {
        match self.as_ref() {
            Collage::Tiles => mosaic.make_collage(),
            Collage::Gather(gs) => {
                mosaic.apply_collage_gather(&gs.iter().map(|g| g.to_tiles(mosaic)).collect_vec())
            }
            Collage::Pick(p, collage) => mosaic.apply_collage_pick(*p, &collage.to_tiles(mosaic)),
            Collage::Cut(c, collage) => {
                mosaic.apply_collage_cut(c.clone(), &collage.to_tiles(mosaic))
            }
        }
    }
}

impl CollageImportCapability for Tile {
    fn to_collage(&self) -> Option<Box<Collage>> {
        if self.component == "Collage".into() {
            Some(tiles())
        } else if self.component == "CollageGather".into() {
            Some(gather(
                self.iter()
                    .get_arrows()
                    .include_component("CollageGather")
                    .get_targets()
                    .map(|t| t.to_collage().unwrap())
                    .collect_vec(),
            ))
        } else if self.component == "CollagePick".into() {
            let p = self.source();
            let mq = p.to_collage().unwrap();
            match self.get("self").as_u8() {
                0 => Some(arrows_from(mq)),
                1 => Some(descriptors_from(mq)),
                2 => Some(extensions_from(mq)),
                3 => Some(targets_from(mq)),
                4 => Some(sources_from(mq)),
                _ => None,
            }
        } else if self.component == "CollageCut".into() {
            let p = self.source();
            let mq = p.to_collage().unwrap();
            match self.get("self").as_u8() {
                0 => Some(take_components(&[], mq)),
                1 => Some(leave_components(&[], mq)),
                2 => Some(take_objects(mq)),
                3 => Some(take_arrows(mq)),
                4 => Some(take_descriptors(mq)),
                5 => Some(take_extensions(mq)),
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod collage_tests {
    use crate::internals::{take_arrows, targets_from, tiles, Mosaic};

    use super::{CollageExportCapability, CollageImportCapability};

    #[test]
    fn test_collage_caps() {
        let mosaic = Mosaic::new();
        let mq = targets_from(take_arrows(tiles()));
        let _ = mq.to_tiles(&mosaic);

        println!("{}", mosaic.dot());
    }

    #[test]
    fn test_collage_back() {
        let mosaic = Mosaic::new();
        let mq = targets_from(take_arrows(tiles()));
        let t = mq.to_tiles(&mosaic);
        let c = t.to_collage();
        assert!(c.is_some());
        let c = c.unwrap();
        assert_eq!(format!("{:?}", mq), format!("{:?}", c));
    }
}
