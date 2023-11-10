use std::sync::Arc;

use crate::internals::{
    lifecycle::Lifecycle, mosaic_engine::MosaicEngine, query_iterator::QueryIterator, EngineState,
    EntityId, Tile,
};

use super::{
    indirection::{Indirection, QueryIndirect},
    tiling::Tiling,
};

/// This trait allows for parenting relations between entities
/// Introduces the `Parent` marker component that forms the `P --Parent--> C` relation
/// between two entities `P` (the parent) and `C` (the child). Gives ways to add, query, and
/// remove the `Parent` relation.
pub trait Parenting {
    type Entity;
    /** Returns the id of the parenting relation entity (useful for internal bookkeeping)
    /// As a picture, it's like this:
    ///
    ///     parent -parenting relation-> child
    ///
    /// */
    fn get_parenting_relation(&self, child: &Self::Entity) -> Option<Self::Entity>;
    /// Sets the parent to some child entity returning `Ok(parent)` if the child doesn't already have one,
    /// or `Err(old_parent)` if it does (without changing it)
    fn set_parent(
        &self,
        child: &Self::Entity,
        parent: &Self::Entity,
    ) -> Result<Self::Entity, Self::Entity>;
    /// Gets the parent of a child entity
    fn get_parent(&self, child: &Self::Entity) -> Option<Self::Entity>;
    /// Gets all the children of a parent entity
    fn get_children(&self, parent: &Self::Entity) -> QueryIterator;
    /// Unparents a child and deletes the relation
    fn unparent(&self, child: &Self::Entity);
}

impl Parenting for Arc<EngineState> {
    type Entity = EntityId;

    fn get_parenting_relation(&self, child: &EntityId) -> Option<EntityId> {
        let it = self
            .build_query()
            .with_target(*child)
            .with_component("Parent".into())
            .get();
        assert!(it.len() <= 1);
        it.as_slice().first().cloned()
    }

    fn set_parent(&self, child: &EntityId, parent: &EntityId) -> Result<EntityId, EntityId> {
        if let Some(relation) = self.get_parenting_relation(child) {
            Err(self.get_source(&relation).unwrap())
        } else {
            let _p = self.create_arrow(parent, child, "Parent".into(), vec![]);
            Ok(*parent)
        }
    }

    fn get_parent(&self, child: &EntityId) -> Option<EntityId> {
        self.get_parenting_relation(child)
            .and_then(|p| self.get_source(&p))
    }

    fn get_children(&self, parent: &EntityId) -> QueryIterator {
        (
            self,
            self.build_query()
                .with_source(*parent)
                .with_component("Parent".into())
                .get_targets()
                .as_vec(),
        )
            .into()
    }

    fn unparent(&self, child: &EntityId) {
        if let Some(rel) = self.get_parenting_relation(child) {
            self.destroy_arrow(rel);
        }
    }
}

impl Parenting for Arc<MosaicEngine> {
    type Entity = Tile;

    fn get_parenting_relation(&self, child: &Self::Entity) -> Option<Self::Entity> {
        self.engine_state
            .get_parenting_relation(&child.id())
            .and_then(|b| self.get_tile(b))
    }

    fn set_parent(
        &self,
        child: &Self::Entity,
        parent: &Self::Entity,
    ) -> Result<Self::Entity, Self::Entity> {
        self.engine_state
            .set_parent(&child.id(), &parent.id())
            .map(|b| self.get_tile(b).unwrap())
            .map_err(|b| self.get_tile(b).unwrap())
    }

    fn get_parent(&self, child: &Self::Entity) -> Option<Self::Entity> {
        self.engine_state
            .get_parent(&child.id())
            .and_then(|b| self.get_tile(b))
    }

    fn get_children(&self, parent: &Self::Entity) -> QueryIterator {
        self.engine_state.get_children(&parent.id())
    }

    fn unparent(&self, child: &Self::Entity) {
        self.engine_state.unparent(&child.id())
    }
}

pub trait ParentingQuery {
    fn no_parent_edges(self) -> Self;
}

impl ParentingQuery for QueryIndirect {
    fn no_parent_edges(mut self) -> Self {
        let engine = Arc::clone(&self.query.engine);
        let filter: Box<dyn FnMut(&EntityId) -> bool> = Box::new(move |i: &EntityId| {
            engine
                .get_brick(*i)
                .map(|e| e.component != "Parent".into())
                .unwrap_or(false)
        });

        self.filters.push(filter);
        self
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod parenting_testing {
    use std::sync::Arc;

    use crate::{
        internals::{lifecycle::Lifecycle, EngineState},
        layers::{indirection::Indirection, parenting::ParentingQuery, querying::Querying},
    };

    use super::Parenting;

    fn setup_parenting_engine_state() -> Arc<EngineState> {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Parent: void;");
        engine_state
    }

    #[test]
    fn test_set_parent() {
        let engine_state = setup_parenting_engine_state();
        let _ = engine_state.add_component_types("Parent: void; Object: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ = engine_state.set_parent(&a, &b);

        let storage = engine_state.entity_brick_storage.lock().unwrap();
        let mut storage_vector = storage.iter().collect::<Vec<_>>();
        storage_vector.sort_by_key(|&(key, _)| *key);
        assert_eq!(3, storage.len());

        let (_, last) = storage_vector.last().unwrap();

        assert_eq!(3, last.id);
        assert_eq!(2, last.source);
        assert_eq!(1, last.target);
    }

    #[test]
    fn test_get_parenting_property() {
        let engine_state = setup_parenting_engine_state();
        let _ = engine_state.add_component_types("Parent: void; Object: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ = engine_state.set_parent(&a, &b);
        let p = engine_state.get_parenting_relation(&a);
        assert!(p.is_some());
        assert_eq!(3, p.unwrap());
    }

    #[test]
    fn test_unparent() {
        let engine_state = setup_parenting_engine_state();
        let _ = engine_state.add_component_types("Parent: void; Object: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ = engine_state.set_parent(&a, &b);
        engine_state.unparent(&a);
        let p = engine_state.get_parenting_relation(&a);
        assert!(p.is_none());

        let storage = engine_state.entity_brick_storage.lock().unwrap();
        let mut storage_vector = storage.iter().collect::<Vec<_>>();
        storage_vector.sort_by_key(|&(key, _)| *key);
        assert_eq!(2, storage.len());
    }

    #[test]
    fn test_get_children() {
        let engine_state = setup_parenting_engine_state();
        let _ = engine_state.add_component_types("Parent: void; Object: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let d = engine_state.create_object("Object".into(), vec![]).unwrap();
        let e = engine_state.create_object("Object".into(), vec![]).unwrap();
        for it in &[b, c, d, e] {
            let _ = engine_state.set_parent(it, &a);
        }

        let children = engine_state.get_children(&a);
        assert_eq!(4, children.len());
        for it in &[b, c, d, e] {
            assert!(children.contains(it));
        }
    }

    #[test]
    fn test_multiple_parents() {
        let engine_state = setup_parenting_engine_state();
        let _ = engine_state.add_component_types("Parent: void; Object: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        assert_eq!(Ok(a), engine_state.set_parent(&c, &a));
        assert_eq!(Err(a), engine_state.set_parent(&c, &b));
    }

    #[test]
    fn test_get_parent() {
        let engine_state = setup_parenting_engine_state();
        let _ = engine_state.add_component_types("Parent: void; Object: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ = engine_state.set_parent(&a, &b);
        assert_eq!(Some(b), engine_state.get_parent(&a));
        assert_eq!(None, engine_state.get_parent(&b));
    }

    #[test]
    fn test_indirection_with_parenting() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Parent: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let d = engine_state.create_object("Object".into(), vec![]).unwrap();

        let _p1 = engine_state.set_parent(&a, &c).unwrap();
        let _ba = engine_state
            .create_arrow(&b, &a, "Arrow".into(), vec![])
            .unwrap();
        let _da = engine_state
            .create_arrow(&d, &a, "Arrow".into(), vec![])
            .unwrap();

        println!(
            "{:?}",
            engine_state
                .get_edges(a)
                .build_query()
                //.select_from(engine_state.query_edges(a).as_vec())
                .no_parent_edges()
                .get_sources()
        );
    }
}
