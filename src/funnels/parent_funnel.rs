use std::sync::Arc;

use crate::{
    internals::{either::EitherByAge, Mosaic, MosaicCRUD, Tile},
    iterators::{
        filter_with_component::FilterWithComponent, get_arrows_from::GetArrowsFromTiles,
        get_arrows_into::GetArrowsIntoTiles, get_tiles::GetTilesIterator,
    },
};

pub trait ParentFunnel {
    fn get_parenting_relation(&self, child: &Tile) -> Option<Tile>;
    fn set_parent(&self, child: &Tile, parent: &Tile) -> EitherByAge<Tile>;
    fn get_parent(&self, child: &Tile) -> Option<Tile>;
    fn get_children(&self, parent: &Tile) -> GetTilesIterator;
    fn unparent(&self, child: &Tile);
}

impl ParentFunnel for Arc<Mosaic> {
    fn get_parenting_relation(&self, child: &Tile) -> Option<Tile> {
        let mut it = child
            .iter_with(self)
            .get_arrows_into()
            .filter_component("Parent");

        let parent = it.next();
        assert_eq!(0, it.count());

        parent
    }

    fn set_parent(&self, child: &Tile, parent: &Tile) -> EitherByAge<Tile> {
        if let Some(parenting_relation) = self.get_parenting_relation(child) {
            EitherByAge::Old(parenting_relation)
        } else {
            self.new_arrow(parent, child, "Parent".into());
            EitherByAge::New(parent.clone())
        }
    }

    fn get_parent(&self, child: &Tile) -> Option<Tile> {
        self.get_parenting_relation(child)
            .and_then(|p| self.get(p.source_id()))
    }

    fn get_children(&self, parent: &Tile) -> GetTilesIterator {
        GetTilesIterator::new(
            parent
                .iter_with(self)
                .get_arrows_from()
                .filter_component("Parent"),
            Arc::clone(self),
        )
    }

    fn unparent(&self, child: &Tile) {
        if let Some(rel) = self.get_parenting_relation(child) {
            self.delete_tile(rel);
        }
    }
}