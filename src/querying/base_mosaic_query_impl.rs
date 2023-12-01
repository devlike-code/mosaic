use std::sync::Arc;

use itertools::Itertools;

use crate::{
    capabilities::{Traversal, Traverse},
    internals::{Mosaic, MosaicIO, Tile},
    iterators::{
        component_selectors::ComponentSelectors, tile_filters::TileFilters,
        tile_getters::TileGetters,
    },
    querying::base_mosaic_query::Cut,
};

use super::base_mosaic_query::{Collage, Composite, MosaicCollage};

impl MosaicCollage for Arc<Mosaic> {
    fn apply_collage(
        &self,
        mq: Box<super::base_mosaic_query::Collage>,
        tiles: Option<Vec<Tile>>,
    ) -> std::vec::IntoIter<crate::internals::Tile> {
        let traversal: Traversal = tiles.unwrap_or(self.get_all().collect_vec()).into();
        mq.query(Arc::clone(self), traversal)
    }
}

impl Collage {
    fn query(
        &self,
        mosaic: Arc<Mosaic>,
        traversal: Traversal,
    ) -> std::vec::IntoIter<crate::internals::Tile> {
        use Composite as S;
        use Cut as F;

        match self {
            Collage::Tiles => mosaic.traverse(traversal).get_all(),
            Collage::Gather(S::Arrows, b) => b.query(mosaic, traversal).get_arrows(),
            Collage::Gather(S::Descriptors, b) => b.query(mosaic, traversal).get_descriptors(),
            Collage::Gather(S::Extensions, b) => b.query(mosaic, traversal).get_extensions(),
            Collage::Gather(S::Targets, b) => b.query(mosaic, traversal).get_targets(),
            Collage::Gather(S::Sources, b) => b.query(mosaic, traversal).get_sources(),
            Collage::Filter(F::Include(components), b) => {
                b.query(mosaic, traversal).include_components(components)
            }
            Collage::Filter(F::Exclude(components), b) => {
                b.query(mosaic, traversal).exclude_components(components)
            }
            Collage::Filter(F::Arrows, b) => b.query(mosaic, traversal).filter_arrows(),
            Collage::Filter(F::Objects, b) => b.query(mosaic, traversal).filter_objects(),
            Collage::Filter(F::Descriptors, b) => b.query(mosaic, traversal).filter_descriptors(),
            Collage::Filter(F::Extensions, b) => b.query(mosaic, traversal).filter_extensions(),
        }
    }
}
