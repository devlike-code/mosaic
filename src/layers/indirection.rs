use crate::internals::{EntityId, EngineState};

/// This is an indirection layer that is built on top of the internals.
pub trait Indirection {
    /// Gets the source of a bricked entity
    fn get_source(&self, brick_id: EntityId) -> Option<EntityId>;
    /// Gets the target of a bricked entity
    fn get_target(&self, brick_id: EntityId) -> Option<EntityId>;
}

impl Indirection for EngineState {
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
}


/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod indirection_testing {
    use crate::{internals::EngineState, layers::indirection::Indirection};

    #[test]
    fn test_indirection_get_source() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        assert_eq!(Some(a), engine_state.get_source(a));
        assert_eq!(Some(b), engine_state.get_source(b));
        assert_eq!(Some(a), engine_state.get_source(c));
    }
    
    #[test]
    fn test_indirection_get_target() {
        let engine_state = EngineState::default();
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let c = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        assert_eq!(Some(a), engine_state.get_target(a));
        assert_eq!(Some(b), engine_state.get_target(b));
        assert_eq!(Some(b), engine_state.get_target(c));
    }
}