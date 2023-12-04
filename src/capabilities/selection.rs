use std::{sync::Arc, vec::IntoIter};

use crate::internals::{void, Mosaic, MosaicIO, MosaicTypelevelCRUD, Tile};

use super::GroupingCapability;

pub trait SelectionCapability: GroupingCapability {
    fn make_selection(&self) -> Tile;
    fn fill_selection(&self, selection: &Tile, members: &[Tile]);
    fn get_selection(&self, selection: &Tile) -> IntoIter<Tile>;
}

impl SelectionCapability for Arc<Mosaic> {
    fn make_selection(&self) -> Tile {
        self.new_type("SelectionOwner: unit;").unwrap();
        let owner = self.new_object("SelectionOwner", void());
        self.group("Selection", &owner, &[]);
        owner
    }

    fn fill_selection(&self, selection: &Tile, members: &[Tile]) {
        self.ungroup("Selection", selection);
        self.group("Selection", selection, members);
    }

    fn get_selection(&self, selection: &Tile) -> IntoIter<Tile> {
        self.get_group_members("Selection", selection)
    }
}
