use crate::internals::{MosaicCRUD, Tile};

pub trait TileDeletion: Iterator {
    fn delete(self);
}

impl<I: Iterator<Item = Tile>> TileDeletion for I {
    fn delete(self) {
        for it in self {
            it.mosaic.delete_tile(it.id);
        }
    }
}
