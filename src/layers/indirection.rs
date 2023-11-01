use array_tool::vec::Uniq;

use crate::internals::{EntityId, EngineState, S32};

use super::querying::Querying;

/// This is an indirection layer that is built on top of the internals.
pub trait Indirection : Querying {
    /// Gets the source of a bricked entity
    fn get_source(&self, brick_id: EntityId) -> Option<EntityId>;
    /// Gets the target of a bricked entity
    fn get_target(&self, brick_id: EntityId) -> Option<EntityId>;
    /// Returns whether this arrow is an incoming property (defined by X: X -> Y)
    fn is_incoming_property(&self, brick_id: EntityId) -> bool;
    /// Returns whether this arrow is an outgoing property (defined by X: Y -> X)
    fn is_outgoing_property(&self, brick_id: EntityId) -> bool;
    /// Gets all the entities that either directly have a component, or have it passed through
    /// a property (both incoming and outgoing)
    fn get_with_property(&self, component: S32) -> Vec<EntityId>;
}

impl Indirection for EngineState {
    fn is_incoming_property(&self, brick_id: EntityId) -> bool {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(&brick_id);
        if let Some(brick) = maybe_brick {
            brick.id == brick.source && brick.id != brick.target
        } else {
            false
        }
    }

    fn is_outgoing_property(&self, brick_id: EntityId) -> bool {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(&brick_id);
        if let Some(brick) = maybe_brick {
            brick.id == brick.target && brick.id != brick.source
        } else {
            false
        }
    }

    fn get_source(&self, brick_id: EntityId) -> Option<EntityId> {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(&brick_id);
        if let Some(brick) = maybe_brick {
            Some(brick.source)
        } else {
            None
        }
    }

    fn get_target(&self, brick_id: EntityId) -> Option<EntityId> {
        let storage = self.entity_brick_storage.lock().unwrap();
        let maybe_brick = storage.get(&brick_id);
        if let Some(brick) = maybe_brick {
            Some(brick.target)
        } else {
            None
        }
    }

    fn get_with_property(&self, component: S32) -> Vec<EntityId> {
        self.query_entities().with_component(component).get()
            .elements().iter().map(|&e| { 
                if self.is_incoming_property(e) {
                    self.get_target(e).unwrap()
                } else if self.is_outgoing_property(e) {
                    self.get_source(e).unwrap()
                } else {
                    e
                }
            }).collect::<Vec<_>>().unique()
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod indirection_testing {
    use crate::{internals::EngineState, layers::indirection::Indirection};

    #[test]
    fn test_get_source() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        assert_eq!(Some(a), engine_state.get_source(a));
        assert_eq!(Some(b), engine_state.get_source(b));
        assert_eq!(Some(a), engine_state.get_source(c));
    }
    
    #[test]
    fn test_get_target() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        assert_eq!(Some(a), engine_state.get_target(a));
        assert_eq!(Some(b), engine_state.get_target(b));
        assert_eq!(Some(b), engine_state.get_target(c));
    }

    #[test]
    fn test_get_with_property() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        let _d = engine_state.add_incoming_property(c, "Data".into(), vec![]);    // c
        let e = engine_state.create_arrow(a, b, "Data".into(), vec![]);   // e
        let _f = engine_state.add_incoming_property(a, "Data".into(), vec![]);   // a
        let data = engine_state.get_with_property("Data".into());
        assert_eq!(3, data.len());
        assert!(data.contains(&a));
        assert!(data.contains(&c));
        assert!(data.contains(&e));
    }
}