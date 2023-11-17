use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::iterators::get_targets::GetTargets;
use crate::{
    internals::{
        get_tiles::{GetTiles, GetTilesExtension, GetTilesIterator},
        Mosaic, MosaicCRUD, Tile, TileCommit, Value,
    },
    iterators::{
        get_arrows_from::GetArrowsFromTiles, get_arrows_into::GetArrowsIntoTiles,
        get_descriptors::GetDescriptors, get_sources::GetSourcesExtension,
        include_component::IncludeComponent,
    },
};

pub trait GroupingCapability {
    fn get_group_memberships(&self, tile: &Tile) -> Vec<Tile>;
    fn group(&self, group: &str, owner: &Tile, members: &[&Tile]);
    fn get_group_owner_descriptor(&self, group: &str, tile: &Tile) -> Option<Tile>;
    fn get_group_owner(&self, group: &str, tile: &Tile) -> Option<Tile>;
    fn get_group_members(&self, group: &str, tile: &Tile) -> GetTilesIterator;
    fn ungroup(&self, group: &str, tile: &Tile);
    fn get_existing_owner_descriptor(&self, group: &str, owner: &Tile) -> Option<Tile>;
}

impl GroupingCapability for Arc<Mosaic> {
    fn get_group_owner_descriptor(&self, group: &str, tile: &Tile) -> Option<Tile> {
        if let Some(current_owner_descriptor) = self.get_existing_owner_descriptor(group, tile) {
            Some(current_owner_descriptor)
        } else {
            tile.iter_with(self)
                .get_arrows_into()
                .include_component("Group")
                .map(|s| (s["self"].as_s32(), s))
                .filter(|(c, _)| c == &group.into())
                .map(|(_, t)| t)
                .get_sources_with(self)
                .collect_vec()
                .first()
                .cloned()
        }
    }

    fn get_group_owner(&self, group: &str, tile: &Tile) -> Option<Tile> {
        self.get_group_owner_descriptor(group, tile)
            .and_then(|t| self.get(t.target_id()))
    }

    fn get_group_memberships(&self, tile: &Tile) -> Vec<Tile> {
        tile.iter_with(self)
            .get_arrows_into()
            .include_component("Group")
            .unique_by(|t| t["self"].as_s32())
            .collect_vec()
    }

    fn get_existing_owner_descriptor(&self, group: &str, owner: &Tile) -> Option<Tile> {
        owner
            .iter_with(self)
            .get_descriptors()
            .include_component("GroupOwner")
            .map(|t| (t["self"].as_s32(), t))
            .collect::<HashMap<_, _>>()
            .get(&group.into())
            .cloned()
    }

    fn group(&self, group: &str, owner: &Tile, members: &[&Tile]) {
        if let Some(previous_owner_descriptor) = self.get_existing_owner_descriptor(group, owner) {
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

    fn get_group_members(&self, group: &str, tile: &Tile) -> GetTilesIterator {
        if let Some(owner) = self.get_group_owner_descriptor(group, tile) {
            owner
                .iter_with(self)
                .get_arrows_from()
                .include_component("Group")
                .get_targets()
                .get_tiles()
        } else {
            vec![].into_iter().get_tiles_with(Arc::clone(self))
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

#[cfg(test)]
mod grouping_tests {

    use crate::internals::{Mosaic, MosaicCRUD, MosaicTypelevelCRUD};

    use super::GroupingCapability;

    #[test]
    fn group_owner_test() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Group: s32;").unwrap();

        let o = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let c = mosaic.new_object("DEBUG");
        let d = mosaic.new_object("DEBUG");

        /*
                         /----> b
           o ----group(p) ----> c
                         \----> d

        */

        mosaic.group("Parent", &o, &[&b, &c, &d]);

        let e = mosaic.get_existing_owner_descriptor("Parent", &o).unwrap();
        println!("EXISTING OWNER Descriptor: {:?}", e);

        mosaic.group("Parent2", &o, &[&b, &c, &d]);

        mosaic.group("Parent", &o, &[&b, &d]);

        let p = mosaic.get_group_owner_descriptor("Parent", &b);
        println!("New OWNER Descriptor: {:?}", p);
        assert!(!mosaic.is_tile_valid(&e));
        let c_memberships = mosaic.get_group_memberships(&c);
        println!("group membership {:?}", c_memberships);

        assert!(c_memberships.len() == 1);
        assert_eq!(
            c_memberships.first().unwrap()["self"].as_s32(),
            "Parent2".into()
        );

        assert_eq!(mosaic.get_group_owner("Parent", &b), Some(o));
    }
}
