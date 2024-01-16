use std::{collections::HashSet, sync::Arc, vec::IntoIter};

use itertools::Itertools;

use crate::{
    internals::{par, void, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile},
    iterators::{
        component_selectors::ComponentSelectors, tile_deletion::TileDeletion,
        tile_getters::TileGetters,
    },
};

pub trait SelectionCapability {
    fn make_selection(&self, members: &[Tile]) -> Tile;
    fn update_selection(&self, selection: &Tile, members: &[Tile]);
    fn get_selection(&self, selection: &Tile) -> IntoIter<Tile>;
    fn clear_selection(&self, selection: &Tile);
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

    fn update_selection(&self, owner: &Tile, members: &[Tile]) {
        let old_members: HashSet<Tile> =
            HashSet::from_iter(owner.iter().get_extensions().include_component("Selection"));

        for member in members {
            if !old_members.contains(member) {
                self.new_extension(owner, "Selection", par(member.id as u64));
            }
        }

        for old in old_members {
            if !members.contains(&old) {
                old.iter().delete();
            }
        }
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

    fn clear_selection(&self, selection: &Tile) {
        selection
            .iter()
            .get_extensions()
            .include_component("Selection")
            .delete();
    }
}
