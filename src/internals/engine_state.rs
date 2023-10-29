use std::{sync::{Mutex, MutexGuard}, collections::HashMap};

use super::{datatypes::{S32, ComponentType, EntityId}, sparse_set::SparseSet};

#[derive(Default)]
/// The full state of the engine, with fields that keep the run-time of the full platform.
pub struct EngineState {
    /// The component type index that holds a mapping of all the registered types by their name
    pub component_type_index: Mutex<HashMap<S32, ComponentType>>,    
    
    /// The current entity count for this engine - grows by one every time a new entity is created
    pub entity_counter: Mutex<usize>,
    
    /// The set of all valid entities (those that are alive, undeleted)
    pub valid_entity_set: Mutex<SparseSet>,

    /// The index of all entities that have a certain component
    pub entities_by_component_index: Mutex<HashMap<S32, SparseSet>>,

    /// The index of all entities that have a specific entity as their source
    pub entities_by_source_index: Mutex<HashMap<EntityId, SparseSet>>,

    /// The index of all entities that have a specific entity as their target
    pub entities_by_target_index: Mutex<HashMap<EntityId, SparseSet>>,

    /// The index of all entities that have both specific source and target
    pub entities_by_both_endpoints_index: Mutex<HashMap<(EntityId, EntityId), SparseSet>>,
}

/// Private implementations for engine state
impl EngineState {
    fn get_component_type_index(&self) -> MutexGuard<'_, HashMap<S32, ComponentType>> {
        self.component_type_index.lock().unwrap()
    }
}

/// Public implementations for engine state
impl EngineState {
    pub fn add_component_type(&self, definition: ComponentType) {
        self.component_type_index.lock().unwrap().insert(definition.name().into(), definition);
    }

    pub fn get_component_type(&self, name: S32) -> Option<ComponentType> {
        self.component_type_index.lock().unwrap().get(&name).cloned()
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod engine_state_testing {
    use crate::internals::datatypes::Datatype;

    use super::{ComponentType, EngineState};

    #[test]
    fn test_engine_state_add_component_type() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Foo".into(), aliased: Datatype::EID });

        let component_type_index = engine_state.get_component_type_index();

        assert!(!component_type_index.is_empty());
        assert!(component_type_index.contains_key(&"Foo".into()));
        assert!(component_type_index.get(&"Foo".into()).unwrap().is_alias());
        assert_eq!("Foo", component_type_index.get(&"Foo".into()).unwrap().name());
    }

    #[test]
    fn test_engine_state_get_component_type() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Foo".into(), aliased: Datatype::EID });

        let output = engine_state.get_component_type("Foo".into());
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(output.is_alias());
        assert_eq!("Foo", output.name());
    }
}