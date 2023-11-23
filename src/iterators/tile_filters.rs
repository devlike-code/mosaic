use std::vec::IntoIter;

use itertools::Itertools;

use crate::internals::Tile;

pub trait TileFilters: Iterator {
    fn filter_arrows(self) -> IntoIter<Self::Item>;
    fn filter_descriptors(self) -> IntoIter<Self::Item>;
    fn filter_extensions(self) -> IntoIter<Self::Item>;
    fn filter_loops(self) -> IntoIter<Self::Item>;
    fn filter_objects(self) -> IntoIter<Self::Item>;
}

impl<I> TileFilters for I
where
    I: Iterator<Item = Tile>,
{
    fn filter_arrows(self) -> IntoIter<Self::Item> {
        self.filter(|tile| tile.is_arrow())
            .collect_vec()
            .into_iter()
    }

    fn filter_descriptors(self) -> IntoIter<Self::Item> {
        self.filter(|tile| tile.is_descriptor())
            .collect_vec()
            .into_iter()
    }

    fn filter_extensions(self) -> IntoIter<Self::Item> {
        self.filter(|tile| tile.is_extension())
            .collect_vec()
            .into_iter()
    }

    fn filter_loops(self) -> IntoIter<Self::Item> {
        self.filter(|tile| tile.is_loop()).collect_vec().into_iter()
    }

    fn filter_objects(self) -> IntoIter<Self::Item> {
        self.filter(|tile| tile.is_object())
            .collect_vec()
            .into_iter()
    }
}
