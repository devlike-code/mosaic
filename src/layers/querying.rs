use std::slice::Iter;

use crate::internals::{EntityId, S32, Brick, EngineState};

pub struct QueryEntities<'a> {
    engine: &'a EngineState,
    source: Option<EntityId>,
    target: Option<EntityId>,
    component: Option<S32>,
}

pub struct QueryIterator {
    iter: Vec<EntityId>,
}

impl QueryIterator {
    pub fn elements(&self) -> Vec<EntityId> {
        self.iter.clone()
    }
}

impl<'a> IntoIterator for &'a QueryIterator {
    type Item = &'a EntityId;

    type IntoIter = Iter<'a, EntityId>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter.iter()
    }
}

impl Into<QueryIterator> for Vec<EntityId> {
    fn into(self) -> QueryIterator {
        QueryIterator { iter: self }
    }
}

impl<'a> QueryEntities<'a> {
    pub fn new(engine: &'a EngineState) -> QueryEntities<'a> {
        QueryEntities { 
            engine, 
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
        match (self.source, self.target, self.component) {
            (None, None, None) => 
                self.engine.entity_brick_storage.lock().unwrap().keys().cloned().collect(),

            (None, None, Some(comp)) => 
                self.engine.entities_by_component_index.lock().unwrap().get(&comp).map(|set| set.elements().clone()).unwrap_or_default(),

            (None, Some(tgt), None) => 
                self.engine.entities_by_target_index.lock().unwrap().get(&tgt).map(|set| set.elements().clone()).unwrap_or_default(),

            (None, Some(tgt), Some(comp)) => 
                self.engine.entities_by_target_and_component_index.lock().unwrap().get(&(tgt, comp)).map(|set| set.elements().clone()).unwrap_or_default(),
                
            (Some(src), None, None) => 
                self.engine.entities_by_source_index.lock().unwrap().get(&src).map(|set| set.elements().clone()).unwrap_or_default(),

            (Some(src), None, Some(comp)) => 
                self.engine.entities_by_source_and_component_index.lock().unwrap().get(&(src, comp)).map(|set| set.elements().clone()).unwrap_or_default(),

            (Some(src), Some(tgt), None) => 
                self.engine.entities_by_both_endpoints_index.lock().unwrap().get(&(src, tgt)).map(|set| set.elements().clone()).unwrap_or_default(),

            (Some(src), Some(tgt), Some(comp)) => 
                self.engine.entities_by_endpoints_and_component_index.lock().unwrap().get(&(src, tgt, comp)).map(|set| set.elements().clone()).unwrap_or_default(),
        }.into()
    }
}

pub trait Querying {
    fn get(&self, id: EntityId) -> Option<Brick>;
    fn query_entities(&self) -> QueryEntities;
}

impl Querying for EngineState {
    fn get(&self, id: EntityId) -> Option<Brick> {
        self.entity_brick_storage.lock().unwrap().get(&id).cloned()
    }

    fn query_entities(&self) -> QueryEntities {
        QueryEntities::new(self)
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod querying_testing {
    use crate::internals::EngineState;

    use super::Querying;

    #[test]
    fn test_get_source() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let _c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        
        let iter = engine_state.query_entities()
            .with_source(a)
            .get();

        assert_eq!(2, iter.iter.len());
    }

    #[test]
    fn test_get_target() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let _c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        
        let iter = engine_state.query_entities()
            .with_target(b)
            .get();

        assert_eq!(2, iter.iter.len());
    }

    #[test]
    fn test_get_component() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let _c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);

        let iter = engine_state.query_entities()
            .with_component("Arrow".into())
            .get();

        assert_eq!(1, iter.iter.len());
    }
}