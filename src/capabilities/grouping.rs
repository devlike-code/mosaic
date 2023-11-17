use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::{
    internals::{
        get_tiles::{GetTiles, GetTilesExtension, GetTilesIterator},
        Mosaic, MosaicCRUD, Tile, TileCommit, Value,
    },
    iterators::{
        deletion::DeleteTiles, filter_descriptors::FilterDescriptors,
        get_arrows_from::GetArrowsFromTiles, get_arrows_into::GetArrowsIntoTiles,
        include_component::IncludeComponent,
    },
};

pub trait GroupingCapability {
    fn get_group_memberships(&self, tile: &Tile) -> Vec<Tile>;
    fn group(&self, group: &str, owner: &Tile, members: &[&Tile]);
    fn get_group_owner(&self, group: &str, tile: &Tile) -> Option<Tile>;
    fn get_group_members(&self, group: &str, tile: &Tile) -> GetTilesIterator;
    fn ungroup(&self, group: &str, tile: &Tile);
}

impl GroupingCapability for Arc<Mosaic> {
    fn get_group_memberships(&self, tile: &Tile) -> Vec<Tile> {
        tile.iter_with(self)
            .get_arrows_into()
            .include_component("Group")
            .unique_by(|t| t["self"].as_s32())
            .collect_vec()
    }

    fn group(&self, group: &str, owner: &Tile, members: &[&Tile]) {
        let existing_owners = owner
            .get_dependents_with(self)
            .filter_descriptors()
            .include_component("GroupOwner")
            .map(|t| (t["self"].as_s32(), t))
            .collect::<HashMap<_, _>>();

        if let Some(previous_owner_descriptor) = existing_owners.get(&group.into()) {
            previous_owner_descriptor
                .iter_with(self)
                .get_arrows_from()
                .delete();
            self.delete_tile(previous_owner_descriptor.id);
        }

        let mut desc = self.new_descriptor(owner, "GroupOwner");
        desc["self"] = Value::S32(group.into());
        self.commit(&desc).unwrap();

        for &member in members {
            let mut group_arrow = self.new_arrow(&desc, member, "Group");
            group_arrow["self"] = Value::S32(group.into());
            self.commit(&group_arrow).unwrap();
        }
    }

    fn get_group_owner(&self, group: &str, tile: &Tile) -> Option<Tile> {
        tile.get_dependents_with(self)
            .get_arrows_into()
            .include_component("Group")
            .map(|t| (t["self"].as_s32(), t))
            .filter(|(c, _)| c == &group.into())
            .map(|(_, t)| t)
            .collect_vec()
            .first()
            .cloned()
    }

    fn get_group_members(&self, group: &str, tile: &Tile) -> GetTilesIterator {
        if let Some(owner) = tile
            .iter_with(self)
            .filter_descriptors()
            .include_component("GroupOwner")
            .filter(|t| t["self"].as_s32() == group.into())
            .collect_vec()
            .first()
        {
            owner
                .iter_with(self)
                .get_arrows_from()
                .include_component("Group")
                .get_tiles()
        } else {
            vec![].into_iter().get_tiles_with(Arc::clone(self))
        }
    }

    fn ungroup(&self, group: &str, tile: &Tile) {
        if let Some(owner) = tile
            .iter_with(self)
            .filter_descriptors()
            .include_component("GroupOwner")
            .filter(|t| t["self"].as_s32() == group.into())
            .collect_vec()
            .first()
        {
            self.delete_tile(owner.id);
        } else if let Some(arrow) = self.get_group_memberships(tile).first() {
            self.delete_tile(arrow.id);
        }
    }
}
