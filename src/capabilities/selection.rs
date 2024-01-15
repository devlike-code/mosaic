use std::{sync::Arc, vec::IntoIter};

use itertools::Itertools;

use crate::{
    internals::{par, void, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile},
    iterators::{component_selectors::ComponentSelectors, tile_getters::TileGetters},
};

use super::GroupingCapability;

pub trait SelectionCapability: GroupingCapability {
    fn make_selection(&self, members: &[Tile]) -> Tile;
    fn get_selection(&self, selection: &Tile) -> IntoIter<Tile>;
}

impl SelectionCapability for Arc<Mosaic> {
    fn make_selection(&self, members: &[Tile]) -> Tile {
        self.new_type("SelectionOwner: unit;").unwrap();
        self.new_type("Selection: u64;").unwrap();

        let owner = self.new_object("SelectionOwner", void());
        for member in members {
            self.new_extension(&owner, "Selection", par(member.id as u64));
        }
        owner
    }

    fn get_selection(&self, selection: &Tile) -> IntoIter<Tile> {
        selection
            .iter()
            .get_extensions()
            .include_component("Selection")
            .map(|t| t.mosaic.get(t.get("self").as_u64() as usize).unwrap())
            .collect_vec()
            .into_iter()
    }
}
