use std::sync::Arc;

use itertools::Itertools;

use crate::{
    internals::{void, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile},
    iterators::{component_selectors::ComponentSelectors, tile_getters::TileGetters},
};

pub trait TupleCapability {
    fn make_tuple(&self, fst: &Tile, snd: &Tile) -> Tile;
    fn get_tuple_first(&self, tuple: &Tile) -> Option<Tile>;
    fn get_tuple_second(&self, tuple: &Tile) -> Option<Tile>;

    fn get_tuple_pair(&self, tuple: &Tile) -> Option<(Tile, Tile)> {
        match (self.get_tuple_first(tuple), self.get_tuple_second(tuple)) {
            (Some(a), Some(b)) => Some((a, b)),
            (a, b) => {
                println!("MISSING VALUE?!?! {:?}, {:?}", a, b);
                None
            }
        }
    }
}

impl TupleCapability for Arc<Mosaic> {
    fn make_tuple(&self, fst: &Tile, snd: &Tile) -> Tile {
        self.new_type("Tuple: unit;").unwrap();
        self.new_type("TupleOwner: unit;").unwrap();
        self.new_type("TupleFirst: unit;").unwrap();
        self.new_type("TupleSecond: unit;").unwrap();
        let tuple_owner = self.new_object("TupleOwner", void());
        self.new_arrow(&tuple_owner, fst, "TupleFirst", void());
        self.new_arrow(&tuple_owner, snd, "TupleSecond", void());
        tuple_owner
    }

    fn get_tuple_first(&self, tuple: &Tile) -> Option<Tile> {
        tuple
            .clone()
            .into_iter()
            .get_arrows_from()
            .include_component("TupleFirst")
            .collect_vec()
            .first()
            .cloned()
    }

    fn get_tuple_second(&self, tuple: &Tile) -> Option<Tile> {
        tuple
            .clone()
            .into_iter()
            .get_arrows_from()
            .include_component("TupleSecond")
            .collect_vec()
            .first()
            .cloned()
    }
}
