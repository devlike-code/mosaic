use std::sync::Arc;

use array_tool::vec::{Intersect, Uniq};
use itertools::Itertools;

use super::{mosaic_engine::MosaicEngine, Tile};

#[derive(Clone, Default)]
/// A query iterator is a thin wrapper around a vector of tiles
pub struct TileIterator {
    pub(crate) engine: Arc<MosaicEngine>,
    pub(crate) tiles: Vec<Tile>,
}

impl std::fmt::Debug for TileIterator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TileIterator")
            .field("tiles", &self.tiles)
            .finish()
    }
}

impl From<(&Arc<MosaicEngine>, Vec<Tile>)> for TileIterator {
    fn from(val: (&Arc<MosaicEngine>, Vec<Tile>)) -> Self {
        TileIterator {
            engine: Arc::clone(val.0),
            tiles: val.1,
        }
    }
}

impl From<(&Arc<MosaicEngine>, Vec<&Tile>)> for TileIterator {
    fn from(val: (&Arc<MosaicEngine>, Vec<&Tile>)) -> Self {
        TileIterator {
            engine: Arc::clone(val.0),
            tiles: val.1.into_iter().cloned().collect_vec(),
        }
    }
}

impl From<(Arc<MosaicEngine>, Vec<Tile>)> for TileIterator {
    fn from(val: (Arc<MosaicEngine>, Vec<Tile>)) -> Self {
        TileIterator {
            engine: val.0,
            tiles: val.1,
        }
    }
}

impl<'a> IntoIterator for &'a TileIterator {
    type Item = &'a Tile;

    type IntoIter = std::slice::Iter<'a, Tile>;

    fn into_iter(self) -> Self::IntoIter {
        self.tiles.iter()
    }
}

impl TileIterator {
    /// Wraps around the length of the current iterator
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Wraps around the `sort` function of the underlying vector
    pub fn sort(&mut self) {
        self.tiles.sort();
    }

    /// Returns a slice of the underlying vector
    pub fn as_slice(&self) -> &[Tile] {
        self.tiles.as_slice()
    }

    /// Returns a clone of the underlying vector
    pub fn as_vec(&self) -> Vec<Tile> {
        self.tiles.clone()
    }

    /// Builds a union of this and another iterator
    pub fn union(mut self, other: TileIterator) -> Self {
        self.tiles.extend(other.as_vec());
        self.tiles = self.tiles.unique();
        self
    }

    /// Builds an intersection of this and another iterator
    pub fn intersect(mut self, other: TileIterator) -> Self {
        self.tiles = self.tiles.intersect(other.as_vec());
        self
    }

    pub fn contains(&self, id: &Tile) -> bool {
        self.tiles.contains(id)
    }
}
