use itertools::Itertools;
use std::vec::IntoIter;

use crate::internals::{MosaicIO, Tile};

use super::tile_filters::TileFilters;

pub trait TileGetters: Iterator {
    fn get_dependents(self) -> IntoIter<Self::Item>;
    fn get_objects(self) -> IntoIter<Self::Item>;
    fn get_arrows(self) -> IntoIter<Self::Item>;
    fn get_loops(self) -> IntoIter<Self::Item>;
    fn get_descriptors(self) -> IntoIter<Self::Item>;
    fn get_extensions(self) -> IntoIter<Self::Item>;
    fn get_sources(self) -> IntoIter<Self::Item>;
    fn get_targets(self) -> IntoIter<Self::Item>;
    fn get_arrows_into(self) -> IntoIter<Self::Item>;
    fn get_arrows_from(self) -> IntoIter<Self::Item>;
}

impl<I> TileGetters for I
where
    I: Iterator<Item = Tile>,
{
    fn get_dependents(self) -> IntoIter<Tile> {
        self.into_iter()
            .flat_map(|tile| {
                let tile_storage = tile.mosaic.tile_registry.lock().unwrap();

                tile.mosaic
                    .dependent_ids_map
                    .lock()
                    .unwrap()
                    .get_all(&tile.id)
                    .filter_map(|id| tile_storage.get(id))
                    .cloned()
                    .collect_vec()
            })
            .collect_vec()
            .into_iter()
    }

    fn get_objects(self) -> IntoIter<Self::Item> {
        self.get_dependents().filter_objects()
    }

    fn get_arrows(self) -> IntoIter<Self::Item> {
        self.get_dependents().filter_arrows()
    }

    fn get_loops(self) -> IntoIter<Self::Item> {
        self.get_dependents().filter_loops()
    }

    fn get_descriptors(self) -> IntoIter<Self::Item> {
        self.get_dependents().filter_descriptors()
    }

    fn get_extensions(self) -> IntoIter<Self::Item> {
        self.get_dependents().filter_extensions()
    }

    fn get_sources(self) -> IntoIter<Self::Item> {
        self.flat_map(|t| t.mosaic.get(t.source_id()))
            .collect_vec()
            .into_iter()
    }

    fn get_targets(self) -> IntoIter<Self::Item> {
        self.flat_map(|t| t.mosaic.get(t.target_id()))
            .collect_vec()
            .into_iter()
    }

    fn get_arrows_into(self) -> IntoIter<Self::Item> {
        self.into_iter()
            .flat_map(|tile| {
                let tile_storage = tile.mosaic.tile_registry.lock().unwrap();
                let id = tile.id;
                tile.mosaic
                    .dependent_ids_map
                    .lock()
                    .unwrap()
                    .get_all(&id)
                    .filter_map(|id| tile_storage.get(id))
                    .filter(|tile| tile.is_arrow() && tile.target_id() == id)
                    .cloned()
                    .unique()
                    .collect_vec()
            })
            .collect_vec()
            .into_iter()
    }

    fn get_arrows_from(self) -> IntoIter<Self::Item> {
        self.into_iter()
            .flat_map(|tile| {
                let tile_storage = tile.mosaic.tile_registry.lock().unwrap();
                let id = tile.id;
                tile.mosaic
                    .dependent_ids_map
                    .lock()
                    .unwrap()
                    .get_all(&id)
                    .filter_map(|id| tile_storage.get(id))
                    .filter(|tile| tile.is_arrow() && tile.source_id() == id)
                    .cloned()
                    .unique()
                    .collect_vec()
            })
            .collect_vec()
            .into_iter()
    }
}
