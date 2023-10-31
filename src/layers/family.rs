
use crate::internals::{EntityId, EngineState};

pub trait Family {
    fn get_parenting_property(&self, child: EntityId) -> Option<EntityId>;
    fn set_parent(&self, child: EntityId, parent: EntityId);
    fn get_parent(&self, child: EntityId) -> Option<EntityId>;
    fn get_children(&self, parent: EntityId) -> Vec<EntityId>;
    fn unparent(&self, child: EntityId);
}

impl Family for EngineState {
    fn get_parenting_property(&self, child: EntityId) -> Option<EntityId> {
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

    fn set_parent(&self, child: EntityId, parent: EntityId) {
        self.create_arrow(parent, child, "Parent".into(), vec![]);
    }

    fn get_parent(&self, child: EntityId) -> Option<EntityId> {
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

    fn get_children(&self, parent: EntityId) -> Vec<EntityId> {
        let storage = self.entity_brick_storage.lock().unwrap();
        let parent_index = self.entities_by_source_and_component_index.lock().unwrap();
        if let Some(parents) = parent_index.get(&(parent, "Parent".into())) {
            parents.into_iter().map(|e| &storage.get(e).unwrap().target).cloned().collect()
        } else {
            vec![]
        }
    }

    fn unparent(&self, child: EntityId) {
        if let Some(rel) = self.get_parenting_property(child) {
            self.delete_property(rel);
        }
    }
}


/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod family_testing {
    use crate::internals::EngineState;

    use super::Family;

    #[test]
    fn test_family_set_parent() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        engine_state.set_parent(a, b);
        
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
    fn test_family_get_parenting_property() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        engine_state.set_parent(a, b);
        let p = engine_state.get_parenting_property(a);
        assert!(p.is_some());
        assert_eq!(3, p.unwrap());
    }

    #[test]
    fn test_family_unparent() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        engine_state.set_parent(a, b);
        engine_state.unparent(a);
        let p = engine_state.get_parenting_property(a);
        assert!(p.is_none());

        let storage = engine_state.entity_brick_storage.lock().unwrap();
        let mut storage_vector = storage.iter().collect::<Vec<_>>();
        storage_vector.sort_by_key(|&(key, _)| *key);
        assert_eq!(2, storage.len());
    }

    #[test]
    fn test_family_get_children() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_object();
        let d = engine_state.create_object();
        let e = engine_state.create_object();
        for it in &[b, c, d, e] {
            engine_state.set_parent(*it, a);
        }
        
        let children = engine_state.get_children(a);
        assert_eq!(4, children.len());
        for it in &[b, c, d, e] {
            assert!(children.contains(it));
        }
    }
    
}