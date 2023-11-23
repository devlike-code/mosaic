use std::vec::IntoIter;
use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::internals::{self_val, Logging, MosaicIO, TileFieldGetter};
use crate::internals::{Mosaic, MosaicCRUD, Tile, Value};
use crate::iterators::component_selectors::ComponentSelectors;
use crate::iterators::tile_getters::TileGetters;

pub trait GroupingCapability {
    fn get_group_memberships(&self, tile: &Tile) -> Vec<Tile>;
    fn group(&self, group: &str, owner: &Tile, members: &[&Tile]);
    fn add_group_member(&self, group: &str, owner: &Tile, member: &Tile) -> anyhow::Result<()>;
    fn get_group_owner_descriptor(&self, group: &str, tile: &Tile) -> Option<Tile>;
    fn get_group_owner(&self, group: &str, tile: &Tile) -> Option<Tile>;
    fn get_group_members(&self, group: &str, tile: &Tile) -> IntoIter<Tile>;
    fn ungroup(&self, group: &str, tile: &Tile);
}

fn get_existing_owner_descriptor(group: &str, owner: &Tile) -> Option<Tile> {
    owner
        .clone()
        .into_iter()
        .get_descriptors()
        .include_component("GroupOwner")
        .map(|t| (t.get("self").as_s32(), t))
        .collect::<HashMap<_, _>>()
        .get(&group.into())
        .cloned()
}

impl GroupingCapability for Arc<Mosaic> {
    fn get_group_memberships(&self, tile: &Tile) -> Vec<Tile> {
        tile.clone()
            .into_iter()
            .get_arrows_into()
            .include_component("Group")
            .unique_by(|t| t.get("self").as_s32())
            .collect_vec()
    }

    fn group(&self, group: &str, owner: &Tile, members: &[&Tile]) {
        if let Some(previous_owner_descriptor) = get_existing_owner_descriptor(group, owner) {
            self.delete_tile(previous_owner_descriptor.id);
        }

        let desc = self.new_descriptor(owner, "GroupOwner", self_val(Value::S32(group.into())));

        for &member in members {
            self.new_arrow(&desc, member, "Group", self_val(Value::S32(group.into())));
        }
    }

    fn add_group_member(&self, group: &str, owner: &Tile, member: &Tile) -> anyhow::Result<()> {
        if let Some(owner_descriptor) = get_existing_owner_descriptor(group, owner) {
            self.new_arrow(
                &owner_descriptor,
                member,
                "Group",
                self_val(Value::S32(group.into())),
            );
            Ok(())
        } else {
            format!(
                "Cannot add group member {:?} to non-existing group {} on tile {:?}",
                member, group, owner
            )
            .to_error()
        }
    }

    fn get_group_owner_descriptor(&self, group: &str, tile: &Tile) -> Option<Tile> {
        if let Some(current_owner_descriptor) = get_existing_owner_descriptor(group, tile) {
            Some(current_owner_descriptor)
        } else {
            tile.clone()
                .into_iter()
                .get_arrows_into()
                .include_component("Group")
                .map(|s| (s.get("self").as_s32(), s))
                .filter(|(c, _)| c == &group.into())
                .map(|(_, t)| t)
                .get_sources()
                .collect_vec()
                .first()
                .cloned()
        }
    }

    fn get_group_owner(&self, group: &str, tile: &Tile) -> Option<Tile> {
        self.get_group_owner_descriptor(group, tile)
            .and_then(|t| self.get(t.target_id()))
    }

    fn get_group_members(&self, group: &str, tile: &Tile) -> IntoIter<Tile> {
        if let Some(owner) = self.get_group_owner_descriptor(group, tile) {
            owner
                .into_iter()
                .get_arrows_from()
                .include_component("Group")
                .get_targets()
        } else {
            vec![].into_iter()
        }
    }

    fn ungroup(&self, group: &str, tile: &Tile) {
        if let Some(owner) = self.get_group_owner_descriptor(group, tile) {
            self.delete_tile(owner.id);
        } else if let Some(arrow) = self.get_group_memberships(tile).first() {
            self.delete_tile(arrow.id);
        }
    }
}
