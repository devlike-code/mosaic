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

use super::base_mosaic_query::{Collage, MosaicCollage, Pick};

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
        use Cut as F;
        use Pick as S;

        match self {
            Collage::Tiles => mosaic.traverse(traversal).get_all(),
            Collage::Pick(S::Arrows, b) => b.query(mosaic, traversal).get_arrows(),
            Collage::Pick(S::Descriptors, b) => b.query(mosaic, traversal).get_descriptors(),
            Collage::Pick(S::Extensions, b) => b.query(mosaic, traversal).get_extensions(),
            Collage::Pick(S::Targets, b) => b.query(mosaic, traversal).get_targets(),
            Collage::Pick(S::Sources, b) => b.query(mosaic, traversal).get_sources(),
            Collage::Cut(F::Include(components), b) => {
                b.query(mosaic, traversal).include_components(components)
            }
            Collage::Cut(F::Exclude(components), b) => {
                b.query(mosaic, traversal).exclude_components(components)
            }
            Collage::Cut(F::Arrows, b) => b.query(mosaic, traversal).filter_arrows(),
            Collage::Cut(F::Objects, b) => b.query(mosaic, traversal).filter_objects(),
            Collage::Cut(F::Descriptors, b) => b.query(mosaic, traversal).filter_descriptors(),
            Collage::Cut(F::Extensions, b) => b.query(mosaic, traversal).filter_extensions(),
            Collage::Gather(bs) => bs
                .iter()
                .map(|b| b.query(Arc::clone(&mosaic), traversal.clone()))
                .fold(vec![].into_iter(), |all, next| {
                    all.chain(next).unique().collect_vec().into_iter()
                }),
        }
    }
}
