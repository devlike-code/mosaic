use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::{
    internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile},
    iterators::{component_selectors::ComponentSelectors, tile_getters::TileGetters},
};

use super::{GroupingCapability, TupleCapability};

pub trait DictionaryCapability: GroupingCapability {
    fn make_dictionary(&self) -> Tile;
    fn add_dictionary_entry(&self, dict: &Tile, key: &Tile, value: &Tile);
    fn get_dictionary_value(&self, dict: &Tile, key: &Tile) -> Option<Tile>;
    fn get_dictionary_entries(&self, dict: &Tile) -> HashMap<Tile, Tile>;
}

impl DictionaryCapability for Arc<Mosaic> {
    fn make_dictionary(&self) -> Tile {
        self.new_type("Dictionary: unit;").unwrap();
        self.new_type("DictionaryEntry: unit;").unwrap();

        self.new_object("Dictionary", default_vals())
    }

    fn add_dictionary_entry(&self, dict: &Tile, key: &Tile, value: &Tile) {
        let entry = self.make_tuple(key, value);
        self.new_arrow(dict, &entry, "DictionaryEntry", default_vals());
    }

    fn get_dictionary_value(&self, dict: &Tile, key: &Tile) -> Option<Tile> {
        for tuple in dict
            .clone()
            .into_iter()
            .get_arrows_from()
            .include_component("DictionaryEntry")
            .get_targets()
        {
            if let Some(k) = self.get_tuple_first(&tuple) {
                if &k == key {
                    return self.get_tuple_second(&tuple);
                }
            }
        }

        None
    }

    fn get_dictionary_entries(&self, dict: &Tile) -> HashMap<Tile, Tile> {
        HashMap::from_iter(
            dict.clone()
                .into_iter()
                .get_arrows_from()
                .get_targets()
                .get_arrows_from()
                .include_component("DictionaryEntry")
                .get_targets()
                .filter_map(|tuple| self.get_tuple_pair(&tuple))
                .map(|(a, b)| (a.target(), b.target()))
                .collect_vec(),
        )
    }
}
