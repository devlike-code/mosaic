
use crate::internals::{EntityId, EngineState};

use super::{indirection::Indirection, query_iterator::QueryIterator};

/// This trait allows for parenting relations between entities
/// Introduces the `Parent` marker component that forms the `P --Parent--> C` relation
/// between two entities `P` (the parent) and `C` (the child). Gives ways to add, query, and
/// remove the `Parent` relation.
pub trait Parenting {
    /** Returns the id of the parenting relation entity (useful for internal bookkeeping)
    /// As a picture, it's like this:
    /// 
    ///     parent -parenting relation-> child
    /// 
    /// */
    fn get_parenting_relation(&self, child: EntityId) -> Option<EntityId>;
    /// Sets the parent to some child entity returning `Ok(parent)` if the child doesn't already have one,
    /// or `Err(old_parent)` if it does (without changing it)
    fn set_parent(&self, child: EntityId, parent: EntityId) -> Result<EntityId, EntityId>;
    /// Gets the parent of a child entity
    fn get_parent(&self, child: EntityId) -> Option<EntityId>;
    /// Gets all the children of a parent entity
    fn get_children(&self, parent: EntityId) -> QueryIterator;
    /// Unparents a child and deletes the relation
    fn unparent(&self, child: EntityId);
}

impl Parenting for EngineState {
    fn get_parenting_relation(&self, child: EntityId) -> Option<EntityId> {
        let it = self.query().with_target(child).with_component("Parent".into()).get();
        assert!(it.len() <= 1);
        it.as_slice().first().cloned()
    }

    fn set_parent(&self, child: EntityId, parent: EntityId) -> Result<EntityId, EntityId> {
        if let Some(relation) = self.get_parenting_relation(child) {
            Err(self.get_source(relation).unwrap())
        } else {
            let _ = self.create_arrow(parent, child, "Parent".into(), vec![]);
            Ok(parent)
        }
    }

    fn get_parent(&self, child: EntityId) -> Option<EntityId> {
        self.get_parenting_relation(child).and_then(|p| self.get_source(p))
    }

    fn get_children(&self, parent: EntityId) -> QueryIterator {
        self.query().with_source(parent).with_component("Parent".into()).get_targets().as_vec().into()
    }

    fn unparent(&self, child: EntityId) {
        if let Some(rel) = self.get_parenting_relation(child) {
            self.destroy_arrow(rel);
        }
    }
}


/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod parenting_testing {
    use crate::internals::EngineState;

    use super::Parenting;

    fn setup_parenting_engine_state() -> EngineState {
        let engine_state = EngineState::default();
        let _ = engine_state.add_component_types("Parent: void;");
        engine_state
    }

    #[test]
    fn test_set_parent() {
        let engine_state = setup_parenting_engine_state();
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _ = engine_state.set_parent(a, b);
        
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
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _ = engine_state.set_parent(a, b);
        let p = engine_state.get_parenting_relation(a);
        assert!(p.is_some());
        assert_eq!(3, p.unwrap());
    }

    #[test]
    fn test_unparent() {
        let engine_state = setup_parenting_engine_state();
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _ = engine_state.set_parent(a, b);
        engine_state.unparent(a);
        let p = engine_state.get_parenting_relation(a);
        assert!(p.is_none());

        let storage = engine_state.entity_brick_storage.lock().unwrap();
        let mut storage_vector = storage.iter().collect::<Vec<_>>();
        storage_vector.sort_by_key(|&(key, _)| *key);
        assert_eq!(2, storage.len());
    }

    #[test]
    fn test_get_children() {
        let engine_state = setup_parenting_engine_state();
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let c = engine_state.create_object_raw("Object".into(), vec![]);
        let d = engine_state.create_object_raw("Object".into(), vec![]);
        let e = engine_state.create_object_raw("Object".into(), vec![]);
        for it in &[b, c, d, e] {
            let _ = engine_state.set_parent(*it, a);
        }
        
        let children = engine_state.get_children(a);
        assert_eq!(4, children.len());
        for it in &[b, c, d, e] {
            assert!(children.contains(it));
        }
    }

    #[test]
    fn test_multiple_parents() {
        let engine_state = setup_parenting_engine_state();
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let c = engine_state.create_object_raw("Object".into(), vec![]);
        assert_eq!(Ok(a), engine_state.set_parent(c, a));
        assert_eq!(Err(a), engine_state.set_parent(c, b));
    }
    
    #[test]
    fn test_get_parent() {
        let engine_state = setup_parenting_engine_state();
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _ = engine_state.set_parent(a, b);
        assert_eq!(Some(b), engine_state.get_parent(a));
        assert_eq!(None, engine_state.get_parent(b));
    }
}