use std::sync::Arc;

use itertools::Itertools;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct JustTileIterator {
    mosaic: Arc<Mosaic>,
    item: Vec<Tile>,
}

impl JustTileIterator {
    pub fn new(tile: Option<Tile>, mosaic: Arc<Mosaic>) -> Self {
        JustTileIterator {
            mosaic: Arc::clone(&mosaic),
            item: tile.into_iter().collect_vec(),
        }
    }
}

impl WithMosaic for JustTileIterator {
    fn get_mosaic(&self) -> Arc<Mosaic> {
        Arc::clone(&self.mosaic)
    }
}

impl Iterator for JustTileIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.item.pop()
    }
}
