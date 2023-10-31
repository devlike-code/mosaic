
use crate::internals::{EntityId, EngineState};

use super::indirection::Indirection;

/// This trait allows for parenting relations between entities
/// Introduces the `Parent` marker component that forms the `P --Parent--> C` relation
/// between two entities `P` (the parent) and `C` (the child). Gives ways to add, query, and
/// remove the `Parent` relation.
pub trait Parenting {
    /// Returns the id of the parenting relation entity (useful for internal bookkeeping)
    fn get_parenting_relation(&self, child: EntityId) -> Option<EntityId>;
    /// Sets the parent to some child entity returning `Ok(parent)` if the child doesn't already have one,
    /// or `Err(old_parent)` if it does (without changing it)
    fn set_parent(&self, child: EntityId, parent: EntityId) -> Result<EntityId, EntityId>;
    /// Gets the parent of a child entity
    fn get_parent(&self, child: EntityId) -> Option<EntityId>;
    /// Gets all the children of a parent entity
    fn get_children(&self, parent: EntityId) -> Vec<EntityId>;
    /// Unparents a child and deletes the relation
    fn unparent(&self, child: EntityId);
}

impl Parenting for EngineState {
    fn get_parenting_relation(&self, child: EntityId) -> Option<EntityId> {
        let parent_index = self.entities_by_target_and_component_index.lock().unwrap();
        if let Some(parents) = parent_index.get(&(child, "Parent".into())) {
            if parents.len() > 0 {
                assert_eq!(1, parents.len());
                parents.elements().get(0).cloned()
            } else {
                None
            }
        } else {
            None
        }
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

    fn get_children(&self, parent: EntityId) -> Vec<EntityId> {
        let index = self.entities_by_source_and_component_index.lock().unwrap();
        if let Some(parent_relation) = index.get(&(parent, "Parent".into())) {
            parent_relation.into_iter().map(|e| self.get_target(*e).unwrap()).collect()
        } else {
            vec![]
        }
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
mod family_testing {
    use crate::internals::EngineState;

    use super::Parenting;

    #[test]
    fn test_parenting_set_parent() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
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
    fn test_parenting_get_parenting_property() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let _ = engine_state.set_parent(a, b);
        let p = engine_state.get_parenting_relation(a);
        assert!(p.is_some());
        assert_eq!(3, p.unwrap());
    }

    #[test]
    fn test_parenting_unparent() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
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
    fn test_parenting_get_children() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_object();
        let d = engine_state.create_object();
        let e = engine_state.create_object();
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
    fn test_parenting_multiple_parents() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_object();
        assert_eq!(Ok(a), engine_state.set_parent(c, a));
        assert_eq!(Err(a), engine_state.set_parent(c, b));
    }
    
    #[test]
    fn test_parenting_get_parent() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let _ = engine_state.set_parent(a, b);
        assert_eq!(Some(b), engine_state.get_parent(a));
        assert_eq!(None, engine_state.get_parent(b));
    }
}