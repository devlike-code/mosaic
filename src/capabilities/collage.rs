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
        all_tiles, arrows_from, descriptors_from, extensions_from, gather, leave_components, par,
        sources_from, take_arrows, take_components, take_descriptors, take_extensions,
        take_objects, targets_from, tiles, void, Collage, Cut, Mosaic, MosaicCRUD, MosaicIO,
        MosaicTypelevelCRUD, Pick, Tile,
    },
    iterators::{component_selectors::ComponentSelectors, tile_getters::TileGetters},
};

use super::{ArchetypeSubject, StringCapability};

pub trait CollageCapability {
    fn make_collage(&self, tiles: Option<Vec<Tile>>) -> Tile;
    fn apply_collage_pick(&self, pick: Pick, target: &Tile) -> Tile;
    fn apply_collage_gather(&self, subs: &[Tile]) -> Tile;
    fn apply_collage_cut(&self, cut: Cut, target: &Tile) -> Tile;
}

impl CollageCapability for Arc<Mosaic> {
    fn make_collage(&self, tiles: Option<Vec<Tile>>) -> Tile {
        self.new_type("Collage: unit;").unwrap();
        self.new_type("CollageTarget: u64;").unwrap();
        self.new_type("CollagePick: u8;").unwrap();
        self.new_type("CollageCut: u8;").unwrap();
        self.new_type("CollageGather: unit;").unwrap();

        let collage = self.new_object("Collage", void());
        for tile in &tiles.unwrap_or_default() {
            self.new_extension(&collage, "CollageTarget", par(tile.id as u64));
        }

        collage
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

pub trait CollageExportCapability {
    fn to_tiles(&self, mosaic: &Arc<Mosaic>) -> Tile;
}

pub trait CollageImportCapability {
    fn to_collage(&self) -> Option<Box<Collage>>;
}

impl CollageExportCapability for Box<Collage> {
    fn to_tiles(&self, mosaic: &Arc<Mosaic>) -> Tile {
        match self.as_ref() {
            Collage::Tiles(None) => mosaic.make_collage(None),
            Collage::Tiles(tiles) => {
                let tiles = tiles
                    .clone()
                    .unwrap()
                    .iter()
                    .map(|v| mosaic.get(*v).unwrap())
                    .collect_vec();
                mosaic.make_collage(Some(tiles))
            }
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
            let p = self.source();
            if let Some(extension) = p.get_component("CollageTarget") {
                let mut tile_id = vec![];

                let value = extension.get("self").as_u64();
                tile_id.push(self.mosaic.get(value.try_into().unwrap()).unwrap());

                if tile_id.is_empty() {
                    Some(all_tiles())
                } else {
                    Some(tiles(tile_id))
                }
            } else {
                Some(all_tiles())
            }
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
            let extensions = p.into_iter().get_extensions().collect_vec();
            let mut strings: Vec<String> = vec![];
            for extension in &extensions {
                let string_extensions = extension
                    .clone()
                    .into_iter()
                    .get_arrows_from()
                    .get_targets()
                    .collect_vec();

                for string in string_extensions {
                    strings.push(self.mosaic.get_string_value(&string).unwrap().clone());
                }
            }

            let mut components = vec![];
            for s in &strings {
                components.push(s.as_str());
            }

            match self.get("self").as_u8() {
                0 => Some(take_components(&components, mq)),
                1 => Some(leave_components(&components, mq)),
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
    use crate::internals::{all_tiles, take_arrows, targets_from, Mosaic};

    use super::{CollageExportCapability, CollageImportCapability};

    #[test]
    fn test_collage_caps() {
        let mosaic = Mosaic::new();
        let mq = targets_from(take_arrows(all_tiles()));
        let _ = mq.to_tiles(&mosaic);

        println!("{}", mosaic.dot());
    }

    #[test]
    fn test_collage_back() {
        let mosaic = Mosaic::new();
        let mq = targets_from(take_arrows(all_tiles()));
        let t = mq.to_tiles(&mosaic);
        let c = t.to_collage();
        assert!(c.is_some());
        let c = c.unwrap();
        assert_eq!(format!("{:?}", mq), format!("{:?}", c));
    }
}
