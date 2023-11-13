use std::sync::Arc;

use itertools::Itertools;

use crate::internals::{
    mosaic_engine::MosaicEngine, query_iterator::QueryIterator, tile_iterator::TileIterator,
    EngineState, EntityId, Tile, S32,
};

use super::tiling::Tiling;

#[derive(Clone)]
/// A simple entities query connected to an engine state and applying one or more filters
pub struct QueryAccess {
    pub(crate) engine: Arc<EngineState>,
    source: Option<EntityId>,
    target: Option<EntityId>,
    component: Option<S32>,
}

impl QueryAccess {
    pub fn new(engine: Arc<EngineState>) -> QueryAccess {
        QueryAccess {
            engine: Arc::clone(&engine),
            source: None,
            target: None,
            component: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_source(mut self, source: EntityId) -> Self {
        self.source = Some(source);
        self
    }

    #[allow(dead_code)]
    pub fn with_target(mut self, target: EntityId) -> Self {
        self.target = Some(target);
        self
    }

    #[allow(dead_code)]
    pub fn with_component(mut self, component: S32) -> Self {
        self.component = Some(component);
        self
    }

    pub fn get(&self) -> QueryIterator {
        let iter = match (self.source, self.target, self.component) {
            (None, None, None) => self
                .engine
                .entity_brick_storage
                .lock()
                .unwrap()
                .keys()
                .cloned()
                .collect(),

            (None, None, Some(comp)) => self
                .engine
                .entities_by_component_index
                .lock()
                .unwrap()
                .get(&comp)
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (None, Some(tgt), None) => self
                .engine
                .entities_by_target_index
                .lock()
                .unwrap()
                .get(&tgt)
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (None, Some(tgt), Some(comp)) => self
                .engine
                .entities_by_target_and_component_index
                .lock()
                .unwrap()
                .get(&(tgt, comp))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), None, None) => self
                .engine
                .entities_by_source_index
                .lock()
                .unwrap()
                .get(&src)
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), None, Some(comp)) => self
                .engine
                .entities_by_source_and_component_index
                .lock()
                .unwrap()
                .get(&(src, comp))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), Some(tgt), None) => self
                .engine
                .entities_by_both_endpoints_index
                .lock()
                .unwrap()
                .get(&(src, tgt))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), Some(tgt), Some(comp)) => self
                .engine
                .entities_by_endpoints_and_component_index
                .lock()
                .unwrap()
                .get(&(src, tgt, comp))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),
        };

        (&self.engine, iter).into()
    }
}

/// Querying is a layer for simple query operations, mostly used in layers higher up
pub(crate) trait Accessing {
    /// Creates a query and passes the engine over to it
    fn query_access(&self) -> QueryAccess;
}

impl Accessing for Arc<EngineState> {
    fn query_access(&self) -> QueryAccess {
        QueryAccess::new(Arc::clone(self))
    }
}

impl Accessing for QueryIterator {
    fn query_access(&self) -> QueryAccess {
        QueryAccess::new(Arc::clone(&self.engine))
    }
}

#[derive(Clone)]
/// A simple entities query connected to an engine state and applying one or more filters
pub struct TileAccess {
    pub(crate) engine: Arc<MosaicEngine>,
    source: Option<Tile>,
    target: Option<Tile>,
    component: Option<S32>,
}

impl TileAccess {
    pub fn new(engine: Arc<MosaicEngine>) -> TileAccess {
        TileAccess {
            engine: Arc::clone(&engine),
            source: None,
            target: None,
            component: None,
        }
    }

    #[allow(dead_code)]
    pub fn with_source(mut self, source: Tile) -> Self {
        self.source = Some(source);
        self
    }

    #[allow(dead_code)]
    pub fn with_target(mut self, target: Tile) -> Self {
        self.target = Some(target);
        self
    }

    #[allow(dead_code)]
    pub fn with_component(mut self, component: S32) -> Self {
        self.component = Some(component);
        self
    }

    pub fn get(&self) -> TileIterator {
        let iter = match (self.source, self.target, self.component) {
            (None, None, None) => self
                .engine
                .engine_state
                .entity_brick_storage
                .lock()
                .unwrap()
                .keys()
                .cloned()
                .collect(),

            (None, None, Some(comp)) => self
                .engine
                .engine_state
                .entities_by_component_index
                .lock()
                .unwrap()
                .get(&comp)
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (None, Some(tgt), None) => self
                .engine
                .engine_state
                .entities_by_target_index
                .lock()
                .unwrap()
                .get(&tgt.id())
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (None, Some(tgt), Some(comp)) => self
                .engine
                .engine_state
                .entities_by_target_and_component_index
                .lock()
                .unwrap()
                .get(&(tgt.id(), comp))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), None, None) => self
                .engine
                .engine_state
                .entities_by_source_index
                .lock()
                .unwrap()
                .get(&src.id())
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), None, Some(comp)) => self
                .engine
                .engine_state
                .entities_by_source_and_component_index
                .lock()
                .unwrap()
                .get(&(src.id(), comp))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), Some(tgt), None) => self
                .engine
                .engine_state
                .entities_by_both_endpoints_index
                .lock()
                .unwrap()
                .get(&(src.id(), tgt.id()))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),

            (Some(src), Some(tgt), Some(comp)) => self
                .engine
                .engine_state
                .entities_by_endpoints_and_component_index
                .lock()
                .unwrap()
                .get(&(src.id(), tgt.id(), comp))
                .map(|set| set.elements().clone())
                .unwrap_or_default(),
        };

        (&self.engine, iter.into_iter().flat_map(|t |self.engine.get_tile(t)).collect_vec()).into()
    }
}

/// Querying is a layer for simple query operations, mostly used in layers higher up
pub(crate) trait TileAccessing {
    /// Creates a query and passes the engine over to it
    fn tile_access(&self) -> TileAccess;
}

impl TileAccessing for Arc<MosaicEngine> {
    fn tile_access(&self) -> TileAccess {
        TileAccess::new(Arc::clone(self))
    }
}

impl TileAccessing for TileIterator {
    fn tile_access(&self) -> TileAccess {
        TileAccess::new(Arc::clone(&self.engine))
    }
}
/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod querying_testing {
    use crate::internals::{lifecycle::Lifecycle, EngineState};

    use super::Accessing;

    #[test]
    fn test_get_source() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _c = engine_state.create_arrow(&a, &b, "Arrow".into(), vec![]);

        let iter = engine_state.query_access().with_source(a).get();

        assert_eq!(2, iter.as_vec().len());
    }

    #[test]
    fn test_get_target() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _c = engine_state.create_arrow(&a, &b, "Arrow".into(), vec![]);

        let iter = engine_state.query_access().with_target(b).get();

        assert_eq!(2, iter.as_vec().len());
    }

    #[test]
    fn test_get_component() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _c = engine_state.create_arrow(&a, &b, "Arrow".into(), vec![]);

        let iter = engine_state
            .query_access()
            .with_component("Arrow".into())
            .get();

        assert_eq!(1, iter.as_vec().len());
    }
}
