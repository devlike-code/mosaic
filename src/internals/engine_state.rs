use std::{sync::Mutex, collections::HashMap};

use super::{datatypes::{S32 as ComponentName, ComponentType, EntityId}, sparse_set::SparseSet, interchange::Brick};

#[derive(Default)]
/// The full state of the engine, with fields that keep the run-time of the full platform.
pub struct EngineState {
    // Type-level index of components

    /// The component type index that holds a mapping of all the registered types by their name
    pub component_type_index: Mutex<HashMap<ComponentName, ComponentType>>,    
    
    // Entity-level book-keeping

    /// The current entity count for this engine - grows by one every time a new entity is created
    pub entity_counter: Mutex<usize>,
    
    /// The set of all valid entities (those that are alive, undeleted)
    pub valid_entity_set: Mutex<SparseSet>,

    /// The storage for all the bricks (id, src, tgt, component, data) tuples that define one brick
    /// (note: bricks have ownership of the information they hold)
    pub entity_brick_storage: Mutex<HashMap<EntityId, Brick>>,

    /// Object index holding a sparseset in which are all entity ids that are of the form (n, n, n)
    pub entity_object_index: Mutex<SparseSet>,

    /// Arrow index holding a sparseset in which are all entity ids that are of the form (n, m, p)
    pub entity_arrow_index: Mutex<SparseSet>,

    /// Arrow index holding a sparseset in which are all entity ids that are of the form (n, m, p)
    pub entity_property_index: Mutex<SparseSet>,

    // Compound book-keeping (join by component, source, target, both endpoints, etc.)

    /// The index of all entities that have a certain component
    pub entities_by_component_index: Mutex<HashMap<ComponentName, SparseSet>>,

    /// The index of all entities that have a specific entity as their source
    pub entities_by_source_index: Mutex<HashMap<EntityId, SparseSet>>,

    /// The index of all entities that have a specific entity as their target
    pub entities_by_target_index: Mutex<HashMap<EntityId, SparseSet>>,

    /// The index of all entities that have both specific source and target
    pub entities_by_both_endpoints_index: Mutex<HashMap<(EntityId, EntityId), SparseSet>>,

    /// The index of all entities that have both specific source and component
    pub entities_by_source_and_component_index: Mutex<HashMap<(EntityId, ComponentName), SparseSet>>,

    /// The index of all entities that have both specific target and component
    pub entities_by_target_and_component_index: Mutex<HashMap<(EntityId, ComponentName), SparseSet>>,

    /// The index of all entities that have both specific source, target, and component
    pub entities_by_endpoints_and_component_index: Mutex<HashMap<(EntityId, EntityId, ComponentName), SparseSet>>,
}

/// Private implementations for engine state
impl EngineState {
    fn get_next_entity_id(&self) -> EntityId {
        let mut counter = self.entity_counter.lock().unwrap();
        *counter += 1;
        *counter
    }

    fn index_entity_by_component(&self, brick: &Brick) {
        let mut index = self.entities_by_component_index.lock().unwrap();
        if !index.contains_key(&brick.component) {
            index.insert(brick.component, SparseSet::default());
        }
        
        index.get_mut(&brick.component).unwrap().add(brick.id);
    }

    fn index_entity_as_object(&self, brick: &Brick) {
        if brick.id == brick.source && brick.source == brick.target {
            self.entity_object_index.lock().unwrap().add(brick.id);
        }
    }

    fn index_entity_as_arrow(&self, brick: &Brick) {
        if brick.id != brick.source && brick.id != brick.target {
            self.entity_arrow_index.lock().unwrap().add(brick.id);
        }
    }

    fn index_entity_as_property(&self, brick: &Brick) {
        if brick.source != brick.target && (brick.id == brick.source || brick.id == brick.target) {
            self.entity_property_index.lock().unwrap().add(brick.id);
        }
    }

    fn index_entity_by_source(&self, brick: &Brick) {
        let mut index = self.entities_by_source_index.lock().unwrap();
        if !index.contains_key(&brick.source) {
            index.insert(brick.source, SparseSet::default());
        }
        
        index.get_mut(&brick.source).unwrap().add(brick.id);
    }

    fn index_entity_by_target(&self, brick: &Brick) {
        let mut index = self.entities_by_target_index.lock().unwrap();
        if !index.contains_key(&brick.target) {
            index.insert(brick.target, SparseSet::default());
        }
        
        index.get_mut(&brick.target).unwrap().add(brick.id);
    }

    fn index_entity_by_both_endpoints(&self, brick: &Brick) {
        let mut index = self.entities_by_both_endpoints_index.lock().unwrap();
        let key = (brick.source, brick.target);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn index_entity_by_source_and_component(&self, brick: &Brick) {
        let mut index = self.entities_by_source_and_component_index.lock().unwrap();
        let key = (brick.source, brick.component);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn index_entity_by_target_and_component(&self, brick: &Brick) {
        let mut index = self.entities_by_target_and_component_index.lock().unwrap();
        let key = (brick.target, brick.component);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn index_entity_by_endpoints_and_component(&self, brick: &Brick) {
        let mut index = self.entities_by_endpoints_and_component_index.lock().unwrap();
        let key = (brick.source, brick.target, brick.component);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn unindex_entity_as_object(&self, brick: &Brick) {
        let mut index = self.entity_object_index.lock().unwrap();
        index.remove(brick.id);
    }

    fn unindex_entity_as_arrow(&self, brick: &Brick) {
        let mut index = self.entity_arrow_index.lock().unwrap();
        index.remove(brick.id);
    }

    fn unindex_entity_as_property(&self, brick: &Brick) {
        let mut index = self.entity_property_index.lock().unwrap();
        index.remove(brick.id);
    }

    fn unindex_entity_by_component(&self, brick: &Brick) {
        let mut index = self.entities_by_component_index.lock().unwrap();
        if index.contains_key(&brick.component) {
            index.get_mut(&brick.component).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_source(&self, brick: &Brick) {
        let mut index = self.entities_by_source_index.lock().unwrap();
        if index.contains_key(&brick.source) {
            index.get_mut(&brick.source).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_target(&self, brick: &Brick) {
        let mut index = self.entities_by_target_index.lock().unwrap();
        if index.contains_key(&brick.target) {
            index.get_mut(&brick.target).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_both_endpoints(&self, brick: &Brick) {
        let mut index = self.entities_by_both_endpoints_index.lock().unwrap();
        let key = (brick.source, brick.target);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_source_and_component(&self, brick: &Brick) {
        let mut index = self.entities_by_source_and_component_index.lock().unwrap();
        let key = (brick.source, brick.component);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_target_and_component(&self, brick: &Brick) {
        let mut index = self.entities_by_target_and_component_index.lock().unwrap();
        let key = (brick.target, brick.component);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_endpoints_and_component(&self, brick: &Brick) {
        let mut index = self.entities_by_endpoints_and_component_index.lock().unwrap();
        let key = (brick.source, brick.target, brick.component);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }
        
    fn add_entity(&self, brick: Brick) {
        self.index_entity_as_object(&brick);
        self.index_entity_as_arrow(&brick);
        self.index_entity_as_property(&brick);
        self.index_entity_by_component(&brick);
        self.index_entity_by_source(&brick);
        self.index_entity_by_target(&brick);
        self.index_entity_by_both_endpoints(&brick);
        self.index_entity_by_source_and_component(&brick);
        self.index_entity_by_target_and_component(&brick);
        self.index_entity_by_endpoints_and_component(&brick);
        self.entity_brick_storage.lock().unwrap().insert(brick.id, brick);
    }

    fn remove_entity(&self, id: EntityId) {
        if let Some(brick) = self.entity_brick_storage.lock().unwrap().remove(&id) {
            self.unindex_entity_as_object(&brick);
            self.unindex_entity_as_arrow(&brick);
            self.unindex_entity_as_property(&brick);
            self.unindex_entity_by_component(&brick);
            self.unindex_entity_by_source(&brick);
            self.unindex_entity_by_target(&brick);
            self.unindex_entity_by_both_endpoints(&brick);
            self.unindex_entity_by_source_and_component(&brick);
            self.unindex_entity_by_target_and_component(&brick);
            self.unindex_entity_by_endpoints_and_component(&brick);
        }
    }
}

/// Public implementations for engine state
impl EngineState {
    /// Register a new component type with the engine
    pub fn add_component_type(&self, definition: ComponentType) {
        self.component_type_index.lock().unwrap().insert(definition.name().into(), definition);
    }

    /// Get a component type by name from the engine
    pub fn get_component_type(&self, name: ComponentName) -> Option<ComponentType> {
        self.component_type_index.lock().unwrap().get(&name).cloned()
    }

    /// Creates a new entity that is categorized as an object (a self-loop from itself to itself)
    /// POST-CONDITION (object-definition): brick.id == brick.source && brick.source == brick.target
    pub fn create_object(&self) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = Brick{ id: index, source: index, target: index, component: "Object".into(), data: vec![] };
        self.add_entity(brick);

        index
    }

    /// Destroy an object entity
    pub fn destroy_object(&self, id: EntityId) {
        self.remove_entity(id);
    }

    /// Creates a new entity that is categorized as an arrow (including non-object self-loop arrows)
    /// Arrows are structural by design and carry a single defining property with them
    /// POST-CONDITION (arrow-definition): brick.id != brick.source && brick.id != brick.target
    pub fn create_arrow(&self, source: EntityId, target: EntityId, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = Brick{ id: index, source, target, component, data };
        self.add_entity(brick);
        
        index
    }

    /// Destroy an arrow entity
    pub fn destroy_arrow(&self, id: EntityId) {
        self.remove_entity(id);
    }

    /// Creates an entity that is categorized as an incoming property, one that has its own entity id as a source
    /// An incoming property `p` of a target `t` looks like this: `p : p -> t`, the property is incoming into the target
    /// POST-CONDITION (incoming-property-definition): brick.id == brick.source && brick.id != brick.target
    pub fn add_incoming_property(&self, target: EntityId, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = Brick{ id: index, source: index, target, component, data };
        self.add_entity(brick);
        
        index
    }

    /// Creates an entity that is categorized as an outgoing property, one that has its own entity id as a target
    /// An outgoing property `p` of a target `t` looks like this: `p : t -> p`, the property is outgoing from the target
    /// POST-CONDITION (outgoing-property-definition): brick.id == brick.target && brick.id != brick.source
    pub fn add_outgoing_property(&self, source: EntityId, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = Brick{ id: index, source, target: index, component, data };
        self.add_entity(brick);
        
        index
    }

    /// Deletes a property entity
    pub fn delete_property(&self, id: EntityId) {
        self.remove_entity(id);
    }
}

impl EngineState {
    
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod engine_state_testing {
    use crate::internals::{datatypes::Datatype, interchange::Brick};

    use super::{ComponentType, EngineState};

    #[test]
    fn test_get_next_entity_id() {
        let engine_state = EngineState::default();
        assert_eq!(1, engine_state.get_next_entity_id());
        assert_eq!(2, engine_state.get_next_entity_id());
        assert_eq!(3, engine_state.get_next_entity_id());
    }

    #[test]
    fn test_add_component_type() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Foo".into(), aliased: Datatype::EID });

        let component_type_index = engine_state.component_type_index.lock().unwrap();

        assert!(!component_type_index.is_empty());
        assert!(component_type_index.contains_key(&"Foo".into()));
        assert!(component_type_index.get(&"Foo".into()).unwrap().is_alias());
        assert_eq!("Foo", component_type_index.get(&"Foo".into()).unwrap().name());
    }

    #[test]
    fn test_get_component_type() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Foo".into(), aliased: Datatype::EID });

        let output = engine_state.get_component_type("Foo".into());
        assert!(output.is_some());
        let output = output.unwrap();
        assert!(output.is_alias());
        assert_eq!("Foo", output.name());
    }

    // Testing that insertion creates the needed index storages
    #[test]
    fn test_add_entity() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Foo".into(), aliased: Datatype::EID });

        let brick = Brick{ id: 1, source: 2, target: 3, component: "Foo".into(), data: vec![] };
        engine_state.add_entity(brick.clone());

        assert!(engine_state.entity_brick_storage.lock().unwrap().contains_key(&brick.id));
        assert!(engine_state.entities_by_component_index.lock().unwrap().contains_key(&brick.component));
        assert!(engine_state.entities_by_component_index.lock().unwrap().get(&brick.component).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_source_index.lock().unwrap().contains_key(&brick.source));
        assert!(engine_state.entities_by_source_index.lock().unwrap().get(&brick.source).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_target_index.lock().unwrap().contains_key(&brick.target));
        assert!(engine_state.entities_by_target_index.lock().unwrap().get(&brick.target).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_both_endpoints_index.lock().unwrap().contains_key(&(brick.source, brick.target)));
        assert!(engine_state.entities_by_both_endpoints_index.lock().unwrap().get(&(brick.source, brick.target)).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_source_and_component_index.lock().unwrap().contains_key(&(brick.source, brick.component)));
        assert!(engine_state.entities_by_source_and_component_index.lock().unwrap().get(&(brick.source, brick.component)).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_target_and_component_index.lock().unwrap().contains_key(&(brick.target, brick.component)));
        assert!(engine_state.entities_by_target_and_component_index.lock().unwrap().get(&(brick.target, brick.component)).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_endpoints_and_component_index.lock().unwrap().contains_key(&(brick.source, brick.target, brick.component)));
        assert!(engine_state.entities_by_endpoints_and_component_index.lock().unwrap().get(&(brick.source, brick.target, brick.component)).unwrap().is_member(brick.id));
    }

    // Testing that, after removal, any index storages remain but the content is correctly freed
    #[test]
    fn test_remove_entity() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Foo".into(), aliased: Datatype::EID });

        let brick = Brick{ id: 1, source: 2, target: 3, component: "Foo".into(), data: vec![] };
        engine_state.add_entity(brick.clone());
        engine_state.remove_entity(brick.id);
        
        assert!(!engine_state.entity_brick_storage.lock().unwrap().contains_key(&brick.id));
        assert!(engine_state.entities_by_component_index.lock().unwrap().contains_key(&brick.component));
        assert!(!engine_state.entities_by_component_index.lock().unwrap().get(&brick.component).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_source_index.lock().unwrap().contains_key(&brick.source));
        assert!(!engine_state.entities_by_source_index.lock().unwrap().get(&brick.source).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_target_index.lock().unwrap().contains_key(&brick.target));
        assert!(!engine_state.entities_by_target_index.lock().unwrap().get(&brick.target).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_both_endpoints_index.lock().unwrap().contains_key(&(brick.source, brick.target)));
        assert!(!engine_state.entities_by_both_endpoints_index.lock().unwrap().get(&(brick.source, brick.target)).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_source_and_component_index.lock().unwrap().contains_key(&(brick.source, brick.component)));
        assert!(!engine_state.entities_by_source_and_component_index.lock().unwrap().get(&(brick.source, brick.component)).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_target_and_component_index.lock().unwrap().contains_key(&(brick.target, brick.component)));
        assert!(!engine_state.entities_by_target_and_component_index.lock().unwrap().get(&(brick.target, brick.component)).unwrap().is_member(brick.id));
        assert!(engine_state.entities_by_endpoints_and_component_index.lock().unwrap().contains_key(&(brick.source, brick.target, brick.component)));
        assert!(!engine_state.entities_by_endpoints_and_component_index.lock().unwrap().get(&(brick.source, brick.target, brick.component)).unwrap().is_member(brick.id));
    }

    #[test]
    fn test_create_object() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Object".into(), aliased: Datatype::VOID });

        let object_id = engine_state.create_object();
        let brick_storage = engine_state.entity_brick_storage.lock().unwrap();

        assert!(brick_storage.contains_key(&object_id));
        if let Some(stored_object) = brick_storage.get(&object_id) {
            assert_eq!(object_id, stored_object.id);
            assert_eq!(object_id, stored_object.source);
            assert_eq!(object_id, stored_object.target);
        }

        assert!(engine_state.entity_object_index.lock().unwrap().is_member(object_id));
    }

    #[test]
    fn test_destroy_object() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Object".into(), aliased: Datatype::VOID });

        let object_id = engine_state.create_object();
        assert!(engine_state.entity_object_index.lock().unwrap().is_member(object_id));
        engine_state.destroy_object(object_id);
        assert!(!engine_state.entity_object_index.lock().unwrap().is_member(object_id));
    }

    #[test]
    fn test_create_arrow() {
        let engine_state = EngineState::default();
        engine_state.add_component_type(ComponentType::Alias { name: "Object".into(), aliased: Datatype::VOID });
        engine_state.add_component_type(ComponentType::Alias { name: "Arrow".into(), aliased: Datatype::VOID });

        let one_id = engine_state.create_object();
        let two_id = engine_state.create_object();
        let three_id = engine_state.create_arrow(one_id, two_id, "Arrow".into(), vec![]);
        let brick_storage = engine_state.entity_brick_storage.lock().unwrap();

        assert!(brick_storage.contains_key(&three_id));
        if let Some(stored_object) = brick_storage.get(&three_id) {
            assert_eq!(three_id, stored_object.id);
            assert_eq!(one_id, stored_object.source);
            assert_eq!(two_id, stored_object.target);
        }
    }
}