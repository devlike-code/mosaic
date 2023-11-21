use std::sync::Arc;

use crate::internals::{Mosaic, Tile, WithMosaic};

pub struct JustTileIterator {
    mosaic: Arc<Mosaic>,
    item: Vec<Tile>,
}

impl JustTileIterator {
    pub fn new(tile: &Tile) -> Self {
        JustTileIterator {
            mosaic: Arc::clone(&tile.mosaic),
            item: vec![tile.clone()],
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
