use std::sync::Arc;

use crate::internals::{MosaicCRUD, Tile, WithMosaic};

pub trait DeleteTiles {
    fn delete(self);
}

impl<I: Iterator<Item = Tile> + WithMosaic> DeleteTiles for I {
    fn delete(self) {
        let mosaic = Arc::clone(&self.get_mosaic());
        for it in self {
            mosaic.delete_tile(it);
        }
    }
}
