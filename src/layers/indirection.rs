use std::{collections::HashSet, sync::Arc};

use array_tool::vec::{Intersect, Uniq};
use itertools::Itertools;

use super::{
    accessing::{Accessing, QueryAccess},
    tiling::Tiling,
};
use crate::internals::{
    mosaic_engine::MosaicEngine, query_iterator::QueryIterator, EngineState, EntityId, Tile, S32,
};

pub type QueryFilter = Box<dyn FnMut(&EntityId) -> bool>;

/// An indirection-layer version of the query, having multiple additional filters
pub struct QueryIndirect {
    pub(crate) query: QueryAccess,
    pub(crate) select: Option<Vec<EntityId>>,
    pub(crate) include_components: Vec<S32>,
    pub(crate) exclude_components: Vec<S32>,
    pub(crate) filters: Vec<QueryFilter>,
}

impl QueryIndirect {
    #[allow(dead_code)]
    pub fn with_source(mut self, source: EntityId) -> Self {
        self.query = self.query.with_source(source);
        self
    }

    #[allow(dead_code)]
    pub fn select_from(mut self, it: Vec<EntityId>) -> Self {
        self.select = Some(it);
        self
    }

    #[allow(dead_code)]
    pub fn with_target(mut self, target: EntityId) -> Self {
        self.query = self.query.with_target(target);
        self
    }

    #[allow(dead_code)]
    pub fn with_component(mut self, component: S32) -> Self {
        self.include_components.push(component);
        self
    }

    #[allow(dead_code)]
    pub fn without_component(mut self, component: S32) -> Self {
        self.exclude_components.push(component);
        self
    }

    pub fn filter<F: 'static + FnMut(&EntityId) -> bool>(mut self, f: F) -> Self {
        self.filters.push(Box::new(f));
        self
    }

    #[allow(dead_code)]
    pub fn no_properties(mut self) -> Self {
        let engine = Arc::clone(&self.query.engine);
        let filter: Box<dyn FnMut(&EntityId) -> bool> =
            Box::new(move |i: &EntityId| !engine.is_property(i));

        self.filters.push(filter);
        self
    }

    #[allow(dead_code)]
    pub fn get(mut self) -> QueryIterator {
        let mut included = self.select.unwrap_or(self.query.get().as_vec());
        for incl in self.include_components {
            let comp = self.query.engine.get_entities_with_property(incl);
            included = included.intersect(comp.as_vec());
        }

        let mut result: HashSet<&usize> = HashSet::from_iter(included.iter().clone());
        for excl in self.exclude_components {
            for &entity in &included {
                if self.query.engine.has_property(&entity, excl) {
                    result.remove(&entity);
                }
            }
        }

        let mut result = result.into_iter().cloned().collect_vec();
        for flt in &mut self.filters {
            result = result.into_iter().filter(flt).collect_vec();
        }

        (self.query.engine, result).into()
    }

    #[allow(dead_code)]
    pub fn get_sources(self) -> QueryIterator {
        Arc::clone(&self.query.engine).get_sources(self.get())
    }

    #[allow(dead_code)]
    pub fn get_targets(self) -> QueryIterator {
        Arc::clone(&self.query.engine).get_targets(self.get())
    }
}

type ComponentName = S32;

/// This is an indirection layer that is built on top of the internals.
pub trait Indirection {
    type Entity;
    /// Gets the source of a bricked entity
    fn get_source(&self, id: &Self::Entity) -> Option<Self::Entity>;
    /// Gets the target of a bricked entity
    fn get_target(&self, id: &Self::Entity) -> Option<Self::Entity>;
    /// Gets an iterator to the sources of an input iterator
    fn get_sources(&self, iter: QueryIterator) -> QueryIterator;
    /// Gets an iterator to the targets of an input iterator
    fn get_targets(&self, iter: QueryIterator) -> QueryIterator;
    /// Returns whether this arrow is an incoming property (defined by X: X -> Y)
    fn is_incoming_property(&self, id: &Self::Entity) -> bool;
    /// Returns whether this arrow is an outgoing property (defined by X: Y -> X)
    fn is_outgoing_property(&self, id: &Self::Entity) -> bool;
    /// Gets all the entities that either directly have a component, or have it passed through
    /// a property (both incoming and outgoing)
    fn get_entities_with_property(&self, component: S32) -> QueryIterator;
    /// Query one or multiple components in inclusion or exclusion
    fn build_query(&self) -> QueryIndirect;
    /// Checks whether the given entity is either an incoming or outgoing property
    fn is_property(&self, id: &Self::Entity) -> bool;
    /// Checks whether the given entity has a component both directly or indirectly
    fn has_property(&self, id: &Self::Entity, component: S32) -> bool;
}

impl Indirection for Arc<EngineState> {
    type Entity = EntityId;

    fn is_incoming_property(&self, id: &EntityId) -> bool {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(id);
        if let Some(brick) = maybe_brick {
            brick.id == brick.source && brick.id != brick.target
        } else {
            false
        }
    }

    fn is_outgoing_property(&self, id: &EntityId) -> bool {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(id);
        if let Some(brick) = maybe_brick {
            brick.id == brick.target && brick.id != brick.source
        } else {
            false
        }
    }

    fn get_source(&self, id: &EntityId) -> Option<EntityId> {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(id);
        maybe_brick.map(|brick| brick.source)
    }

    fn get_target(&self, id: &EntityId) -> Option<EntityId> {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(id);
        maybe_brick.map(|brick| brick.target)
    }

    fn get_sources(&self, iter: QueryIterator) -> QueryIterator {
        (
            self,
            iter.into_iter()
                .flat_map(|e| self.get_source(e))
                .collect_vec(),
        )
            .into()
    }

    fn get_targets(&self, iter: QueryIterator) -> QueryIterator {
        (
            self,
            iter.into_iter()
                .flat_map(|e| self.get_target(e))
                .collect_vec(),
        )
            .into()
    }

    fn get_entities_with_property(&self, component: S32) -> QueryIterator {
        (
            self,
            self.query_access()
                .with_component(component)
                .get()
                .into_iter()
                .map(|e| {
                    if self.is_incoming_property(e) {
                        self.get_target(e).unwrap()
                    } else if self.is_outgoing_property(e) {
                        self.get_source(e).unwrap()
                    } else {
                        *e
                    }
                })
                .collect::<Vec<_>>()
                .unique(),
        )
            .into()
    }

    fn build_query(&self) -> QueryIndirect {
        QueryIndirect {
            select: None,
            query: self.query_access(),
            include_components: vec![],
            exclude_components: vec![],
            filters: vec![],
        }
    }

    fn is_property(&self, id: &Self::Entity) -> bool {
        self.is_incoming_property(id) || self.is_outgoing_property(id)
    }

    fn has_property(&self, id: &Self::Entity, component: S32) -> bool {
        self.get_entities_with_property(component).contains(id)
    }
}

impl Indirection for Arc<MosaicEngine> {
    type Entity = Tile;

    fn get_source(&self, tile: &Self::Entity) -> Option<Tile> {
        self.engine_state
            .get_source(&tile.id())
            .and_then(|t| self.get_tile(t))
    }

    fn get_target(&self, tile: &Self::Entity) -> Option<Self::Entity> {
        self.engine_state
            .get_target(&tile.id())
            .and_then(|t| self.get_tile(t))
    }

    fn get_sources(&self, iter: QueryIterator) -> QueryIterator {
        self.engine_state.get_sources(iter)
    }

    fn get_targets(&self, iter: QueryIterator) -> QueryIterator {
        self.engine_state.get_targets(iter)
    }

    fn is_incoming_property(&self, tile: &Self::Entity) -> bool {
        self.get_tile(tile.id())
            .map(|b| b.is_descriptor())
            .unwrap_or(false)
    }

    fn is_outgoing_property(&self, tile: &Self::Entity) -> bool {
        self.get_tile(tile.id())
            .map(|b| b.is_extension())
            .unwrap_or(false)
    }

    fn get_entities_with_property(&self, component: S32) -> QueryIterator {
        (
            Arc::clone(&self.engine_state),
            self.engine_state
                .get_entities_with_property(component)
                .as_vec(),
        )
            .into()
    }

    fn is_property(&self, tile: &Self::Entity) -> bool {
        tile.is_property()
    }

    fn has_property(&self, tile: &Self::Entity, component: S32) -> bool {
        self.engine_state.has_property(&tile.id(), component)
    }

    fn build_query(&self) -> QueryIndirect {
        self.engine_state.build_query()
    }
}

impl Indirection for QueryIterator {
    type Entity = EntityId;
    fn get_source(&self, id: &EntityId) -> Option<EntityId> {
        self.engine.get_source(id)
    }

    fn get_target(&self, id: &EntityId) -> Option<EntityId> {
        self.engine.get_target(id)
    }

    fn get_sources(&self, iter: QueryIterator) -> QueryIterator {
        self.engine.get_sources(iter)
    }

    fn get_targets(&self, iter: QueryIterator) -> QueryIterator {
        self.engine.get_targets(iter)
    }

    fn is_incoming_property(&self, id: &EntityId) -> bool {
        self.engine.is_incoming_property(id)
    }

    fn is_outgoing_property(&self, id: &EntityId) -> bool {
        self.engine.is_outgoing_property(id)
    }

    fn get_entities_with_property(&self, component: S32) -> QueryIterator {
        self.engine.get_entities_with_property(component)
    }

    fn build_query(&self) -> QueryIndirect {
        self.engine.build_query().select_from(self.elements.clone())
    }

    fn is_property(&self, id: &Self::Entity) -> bool {
        self.engine.is_property(id)
    }

    fn has_property(&self, id: &Self::Entity, component: S32) -> bool {
        self.engine.has_property(id, component)
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod indirection_testing {
    use std::sync::Arc;

    use crate::{
        internals::{lifecycle::Lifecycle, EngineState, EntityId},
        layers::indirection::Indirection,
    };

    #[test]
    fn test_get_source() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        assert_eq!(Some(a), engine_state.get_source(&a));
        assert_eq!(Some(b), engine_state.get_source(&b));
        assert_eq!(Some(a), engine_state.get_source(&c));
    }

    #[test]
    fn test_get_target() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        assert_eq!(Some(a), engine_state.get_target(&a));
        assert_eq!(Some(b), engine_state.get_target(&b));
        assert_eq!(Some(b), engine_state.get_target(&c));
    }

    #[test]
    fn test_get_entities_with_property() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Foo: void; Data: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .create_arrow(&a, &b, "Foo".into(), vec![])
            .unwrap();
        let _d = engine_state
            .add_descriptor(&c, "Data".into(), vec![])
            .unwrap();
        let e = engine_state
            .create_arrow(&a, &b, "Data".into(), vec![])
            .unwrap(); // e
        let _f = engine_state
            .add_descriptor(&a, "Data".into(), vec![])
            .unwrap();
        let data = engine_state.get_entities_with_property("Data".into());
        assert_eq!(3, data.len());
        assert!(data.contains(&a));
        assert!(data.contains(&c));
        assert!(data.contains(&e));
    }

    fn setup_query_tests() -> ([EntityId; 7], Arc<EngineState>) {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Data: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        // C : A ---Arrow---> B
        let c = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        // D : D ---Data----> C
        let d = engine_state
            .add_descriptor(&c, "Data".into(), vec![])
            .unwrap();
        // E : A ---Data----> B
        let e = engine_state
            .create_arrow(&a, &c, "Data".into(), vec![])
            .unwrap();
        // F : F ---Data----> A
        let f = engine_state
            .add_descriptor(&a, "Data".into(), vec![])
            .unwrap();
        // G : E ---Data----> C
        let g = engine_state
            .create_arrow(&e, &c, "Data".into(), vec![])
            .unwrap();
        ([a, b, c, d, e, f, g], engine_state)
    }

    #[test]
    fn test_query_with_source() {
        let ([a, _b, c, _d, e, _f, _g], engine_state) = setup_query_tests();
        let mut all_with_source_a = engine_state.build_query().with_source(a).get();
        all_with_source_a.sort();
        assert_eq!(&[a, c, e], all_with_source_a.as_slice());
    }

    #[test]
    fn test_query_with_component() {
        let ([a, _b, c, _d, e, _f, g], engine_state) = setup_query_tests();
        let mut all_with_comp_data = engine_state
            .build_query()
            .with_component("Data".into())
            .get();
        all_with_comp_data.sort();
        assert_eq!(&[a, c, e, g], all_with_comp_data.as_slice());
    }

    #[test]
    fn test_query_without_component() {
        let ([a, b, _c, d, e, f, g], engine_state) = setup_query_tests();
        let mut all_without_comp_arrow = engine_state
            .build_query()
            .without_component("Arrow".into())
            .get();
        all_without_comp_arrow.sort();
        assert_eq!(&[a, b, d, e, f, g], all_without_comp_arrow.as_slice());
    }

    #[test]
    fn test_query_with_two_components() {
        let ([_a, _b, c, _d, _e, _f, _g], engine_state) = setup_query_tests();
        let mut all_with_comp_arrow_and_data = engine_state
            .build_query()
            .with_component("Arrow".into())
            .with_component("Data".into())
            .get();
        all_with_comp_arrow_and_data.sort();
        assert_eq!(&[c], all_with_comp_arrow_and_data.as_slice());
    }

    #[test]
    fn test_query_without_component_and_no_properties() {
        let ([a, b, _c, _d, e, _f, g], engine_state) = setup_query_tests();
        let mut all_without_comp_arrow_no_prop = engine_state
            .build_query()
            .without_component("Arrow".into())
            .no_properties()
            .get();
        all_without_comp_arrow_no_prop.sort();
        assert_eq!(&[a, b, e, g], all_without_comp_arrow_no_prop.as_slice());
    }

    #[test]
    fn test_query_with_component_and_source() {
        let ([a, _b, c, _d, e, _f, _g], engine_state) = setup_query_tests();
        let mut all_with_source_a_and_data = engine_state
            .build_query()
            .with_source(a)
            .with_component("Data".into())
            .get();
        all_with_source_a_and_data.sort();
        assert_eq!(&[a, c, e], all_with_source_a_and_data.as_slice());
    }

    #[test]
    fn test_query_with_source_without_component() {
        let ([a, _b, _c, _d, e, _f, _g], engine_state) = setup_query_tests();
        let mut all_with_source_a_without_arrow = engine_state
            .build_query()
            .with_source(a)
            .without_component("Arrow".into())
            .get();
        all_with_source_a_without_arrow.sort();
        assert_eq!(&[a, e], all_with_source_a_without_arrow.as_slice());
    }

    #[test]
    fn test_query_with_source_with_one_component_without_another() {
        let ([a, _b, _c, _d, e, _f, _g], engine_state) = setup_query_tests();
        let mut all_with_source_a_and_data_without_arrow = engine_state
            .build_query()
            .with_source(a)
            .with_component("Data".into())
            .without_component("Arrow".into())
            .get();
        all_with_source_a_and_data_without_arrow.sort();
        assert_eq!(&[a, e], all_with_source_a_and_data_without_arrow.as_slice());
    }

    #[test]
    fn test_query_join() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Parent: void; Path: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let d = engine_state.create_object("Object".into(), vec![]).unwrap();
        let e = engine_state.create_object("Object".into(), vec![]).unwrap();

        let _ab = engine_state.create_arrow(&a, &b, "Parent".into(), vec![]);
        let _bc = engine_state.create_arrow(&b, &c, "Path".into(), vec![]);
        let _ad = engine_state.create_arrow(&a, &d, "Parent".into(), vec![]);
        let _cd = engine_state.create_arrow(&c, &d, "Path".into(), vec![]);
        let _de = engine_state.create_arrow(&d, &e, "Parent".into(), vec![]);

        // A ---Parent----> ? -------> C

        // A ---Parent----> ?
        let query_from_a = engine_state
            .build_query()
            .with_source(a)
            .with_component("Parent".into())
            .get_targets();

        // ? -------> C
        let query_to_c = engine_state.build_query().with_target(c).get_sources();

        let join = query_from_a.intersect(query_to_c);
        println!("{:?}", join.as_slice());
        assert_eq!(1, join.len());
        assert_eq!(b, join.as_slice()[0]);
    }
}
