use std::{sync::{Mutex, Arc}, collections::HashMap, ops::Range};

use itertools::Itertools;

use super::{datatypes::{S32 as ComponentName, ComponentType, EntityId}, sparse_set::SparseSet, component_grammar::ComponentParser, Datatype, ComponentField, Value, ToByteArray, Bytesize, lifecycle::Lifecycle};

type FieldName = ComponentName;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// Bricks are the essential building blocks and hold a single component.
/// Every brick contains a single morphism and associated data
pub(crate) struct DataBrick {
    /// Identity of this element
    pub(crate) id: EntityId,
    /// The source element of this morphism
    pub(crate) source: EntityId,
    /// The target element of this morphism
    pub(crate) target: EntityId,
    /// The name of the component representing the data in this morphism
    pub(crate) component: ComponentName,
    /// The actual data carried by the morphism
    pub(crate) data: Vec<u8>,
}

impl DataBrick {
    /// Updates the brick in the engine, lifting any changes into it
    pub(crate) fn update(&self, engine_state: &EngineState) {
        let mut storage = engine_state.entity_brick_storage.lock().unwrap();
        storage.insert(self.id, self.clone());
    }

    /// Refreshes the data from the engine into the brick; it doesn't touch anything other in the brick
    pub(crate) fn refresh(&mut self, engine_state: &EngineState) {
        let storage = engine_state.entity_brick_storage.lock().unwrap();
        self.data = storage.get(&self.id).unwrap_or(self).data.clone();
    }
}

#[derive(Default, Debug)]
/// The full state of the engine, with fields that keep the run-time of the full platform.
pub struct EngineState {
    // Type-level index of components
    // ====================================================================================

    /// The component type index that holds a mapping of all the registered types by their name
    pub component_type_index: Mutex<HashMap<ComponentName, ComponentType>>,    
    
    /// An index of all the component fields' offset and sizes
    pub component_offset_size_index: Mutex<HashMap<(String, FieldName), Range<usize>>>,

    // Entity-level book-keeping
    // ====================================================================================

    /// The current entity count for this engine - grows by one every time a new entity is created
    pub entity_counter: Mutex<usize>,
    
    /// The set of all valid entities (those that are alive, undeleted)
    pub valid_entity_set: Mutex<SparseSet>,

    /// The storage for all the bricks (id, src, tgt, component, data) tuples that define one brick
    /// (note: bricks have ownership of the information they hold)
    pub(crate) entity_brick_storage: Mutex<HashMap<EntityId, DataBrick>>,

    /// Object index holding a sparseset in which are all entity ids that are of the form (n, n, n)
    pub entity_object_index: Mutex<SparseSet>,

    /// Arrow index holding a sparseset in which are all entity ids that are of the form (n, m, p)
    pub entity_arrow_index: Mutex<SparseSet>,

    /// Arrow index holding a sparseset in which are all entity ids that are of the form (n, m, p)
    pub entity_property_index: Mutex<SparseSet>,

    // Compound book-keeping (join by component, source, target, both endpoints, etc.)
    // ====================================================================================

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

impl EngineState {
    pub fn new() -> Arc<EngineState> {
        Arc::new(EngineState::default())
    }
}

/// Private implementations for engine state
impl EngineState {
    fn get_next_entity_id(&self) -> EntityId {
        let storage = self.entity_brick_storage.lock().unwrap();
        let mut counter = self.entity_counter.lock().unwrap();
        
        *counter += 1;
        while storage.contains_key(&*counter) {
            *counter += 1;
        }

        *counter
    }

    fn index_entity_by_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_component_index.lock().unwrap();
        if !index.contains_key(&brick.component) {
            index.insert(brick.component, SparseSet::default());
        }
        
        index.get_mut(&brick.component).unwrap().add(brick.id);
    }

    fn index_entity_as_object(&self, brick: &DataBrick) {
        if brick.id == brick.source && brick.source == brick.target {
            self.entity_object_index.lock().unwrap().add(brick.id);
        }
    }

    fn index_entity_as_arrow(&self, brick: &DataBrick) {
        if brick.id != brick.source && brick.id != brick.target {
            self.entity_arrow_index.lock().unwrap().add(brick.id);
        }
    }

    fn index_entity_as_property(&self, brick: &DataBrick) {
        if brick.source != brick.target && (brick.id == brick.source || brick.id == brick.target) {
            self.entity_property_index.lock().unwrap().add(brick.id);
        }
    }

    fn index_entity_by_source(&self, brick: &DataBrick) {
        let mut index = self.entities_by_source_index.lock().unwrap();
        if !index.contains_key(&brick.source) {
            index.insert(brick.source, SparseSet::default());
        }
        
        index.get_mut(&brick.source).unwrap().add(brick.id);
    }

    fn index_entity_by_target(&self, brick: &DataBrick) {
        let mut index = self.entities_by_target_index.lock().unwrap();
        if !index.contains_key(&brick.target) {
            index.insert(brick.target, SparseSet::default());
        }
        
        index.get_mut(&brick.target).unwrap().add(brick.id);
    }

    fn index_entity_by_both_endpoints(&self, brick: &DataBrick) {
        let mut index = self.entities_by_both_endpoints_index.lock().unwrap();
        let key = (brick.source, brick.target);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn index_entity_by_source_and_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_source_and_component_index.lock().unwrap();
        let key = (brick.source, brick.component);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn index_entity_by_target_and_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_target_and_component_index.lock().unwrap();
        let key = (brick.target, brick.component);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn index_entity_by_endpoints_and_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_endpoints_and_component_index.lock().unwrap();
        let key = (brick.source, brick.target, brick.component);
        if !index.contains_key(&key) {
            index.insert(key, SparseSet::default());
        }
        
        index.get_mut(&key).unwrap().add(brick.id);
    }

    fn unindex_entity_as_object(&self, brick: &DataBrick) {
        let mut index = self.entity_object_index.lock().unwrap();
        index.remove(brick.id);
    }

    fn unindex_entity_as_arrow(&self, brick: &DataBrick) {
        let mut index = self.entity_arrow_index.lock().unwrap();
        index.remove(brick.id);
    }

    fn unindex_entity_as_property(&self, brick: &DataBrick) {
        let mut index = self.entity_property_index.lock().unwrap();
        index.remove(brick.id);
    }

    fn unindex_entity_by_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_component_index.lock().unwrap();
        if index.contains_key(&brick.component) {
            index.get_mut(&brick.component).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_source(&self, brick: &DataBrick) {
        let mut index = self.entities_by_source_index.lock().unwrap();
        if index.contains_key(&brick.source) {
            index.get_mut(&brick.source).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_target(&self, brick: &DataBrick) {
        let mut index = self.entities_by_target_index.lock().unwrap();
        if index.contains_key(&brick.target) {
            index.get_mut(&brick.target).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_both_endpoints(&self, brick: &DataBrick) {
        let mut index = self.entities_by_both_endpoints_index.lock().unwrap();
        let key = (brick.source, brick.target);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_source_and_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_source_and_component_index.lock().unwrap();
        let key = (brick.source, brick.component);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_target_and_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_target_and_component_index.lock().unwrap();
        let key = (brick.target, brick.component);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }

    fn unindex_entity_by_endpoints_and_component(&self, brick: &DataBrick) {
        let mut index = self.entities_by_endpoints_and_component_index.lock().unwrap();
        let key = (brick.source, brick.target, brick.component);
        if index.contains_key(&key) {
            index.get_mut(&key).unwrap().remove(brick.id);
        }
    }
    
    fn add_entity(&self, brick: DataBrick) {
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

    pub(crate) fn get_brick(&self, brick_id: EntityId) -> Option<DataBrick> {
        self
            .entity_brick_storage
            .lock()
            .unwrap()
            .get(&brick_id)
            .cloned()
    }
}

/// Public implementations for engine state
impl EngineState {
    pub(crate) fn get_all_bricks(&self) -> Vec<DataBrick> {
        self.entity_brick_storage.lock().unwrap().values().cloned().collect_vec()
    }

    /// Flatten any notion of a nested component type (`Datatype::COMP`) from the defined type
    fn flatten_component_type(&self, definition: ComponentType) -> Result<ComponentType, String> {
        use ComponentType::*;
        match &definition {
            Alias(ComponentField{ name: _, datatype: Datatype::COMP(other) }) => {
                let other_type = self.get_component_type(other.clone())?;
                Ok(other_type.duplicate_as(definition.name().into()))
            },
            _ => Ok(definition)
        }
    }

    /// Register a new component type with the engine from a component type structure directly (without parsing)
    pub(crate) fn add_raw_component_type(&self, definition: ComponentType) {
        self.component_type_index.lock().unwrap().insert(definition.name().into(), definition.clone());
        
        let mut offset_size_index = self.component_offset_size_index.lock().unwrap();

        let mut offset = 0usize;
        for field in definition.get_fields() {
            let size = field.datatype.bytesize(self);
            let range = offset..offset + size;
            offset_size_index.insert((definition.name().to_string(), field.name), range);
            offset += size;
        }
    }
    
    /// Register a new component type with the engine
    pub fn add_component_types(&self, definition: &str) -> Result<(), String> {
        let types = ComponentParser::parse_all(definition)?;
        for component_type in types {
            self.add_raw_component_type(self.flatten_component_type(component_type)?);
        }
        Ok(())
    }

    /// Checks whether a component by a certain type name exists
    pub fn has_component_type(&self, name: &ComponentName) -> bool {
        self.component_type_index.lock().unwrap().contains_key(name)
    }

    /// Get a component type by name from the engine
    pub fn get_component_type(&self, name: ComponentName) -> Result<ComponentType, String> {
        if self.has_component_type(&name) {
            self.component_type_index.lock().unwrap().get(&name).cloned()
                .ok_or(format!("[Error][engine_state.rs][get_component_type] Component with name '{}' not found", name))
        } else {
            Err(format!("[Error][engine_state.rs][get_component_type] Component with name '{}' not found", name))
        }
    }

    /// Create a specific object under an identifier and return None if it already exists
    pub fn create_specific_object(&self, id: EntityId) -> Option<EntityId> {
        if self.entity_exists(id) { return None; }

        let brick = DataBrick{ id, source: id, target: id, component: "Object".into(), data: vec![] };
        self.add_entity(brick);

        Some(id)
    }

    /// Check whether entity exists
    pub(crate) fn entity_exists(&self, id: EntityId) -> bool {
        self.entity_brick_storage.lock().unwrap().contains_key(&id)
    }

    fn unify_fields_and_values_into_data(&self, component: ComponentName, fields: Vec<Value>) -> Result<Vec<Vec<u8>>, (ComponentField, Value)> {
        let components = self.component_type_index.lock().unwrap();
        let component_type = components.get(&component)
            .ok_or((ComponentField { name: format!("<{}>", component).as_str().into(), datatype: Datatype::VOID }, Value::VOID))?.clone();
        let mut has_error = None;
        let fields = component_type.get_fields().into_iter().zip(fields)
            .map(|(field, datatype_value)| {
                if datatype_value.get_datatype() == field.datatype {
                    Ok(datatype_value.to_byte_array())
                } else {
                    has_error = Some((field.clone(), datatype_value.clone()));
                    Err((field, datatype_value))
                }
            }).collect::<Vec<_>>();
        
        if has_error.is_some() {
            Err(has_error.unwrap())
        } else {
            Ok(fields.iter().map(|e| e.clone().unwrap()).collect())
        }
    }

    /// Creates a new entity that is categorized as an object (a self-loop from itself to itself)
    /// POST-CONDITION (object-definition): brick.id == brick.source && brick.source == brick.target
    pub(crate) fn create_object_raw(&self, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = DataBrick{ id: index, source: index, target: index, component, data };
        self.add_entity(brick);

        index
    }

    /// Destroy an object entity
    pub(crate) fn destroy_object(&self, id: EntityId) {
        self.remove_entity(id);
    }

    /// Creates a new entity that is categorized as an arrow (including non-object self-loop arrows)
    /// Arrows are structural by design and carry a single defining property with them
    /// POST-CONDITION (arrow-definition): brick.id != brick.source && brick.id != brick.target
    pub(crate) fn create_arrow_raw(&self, source: EntityId, target: EntityId, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = DataBrick{ id: index, source, target, component, data };
        self.add_entity(brick);
        
        index
    }

    /// Destroy an arrow entity
    pub(crate) fn destroy_arrow(&self, id: EntityId) {
        self.remove_entity(id);
    }

    /// Creates an entity that is categorized as an incoming property, one that has its own entity id as a source
    /// An incoming property `p` of a target `t` looks like this: `p : p -> t`, the property is incoming into the target
    /// POST-CONDITION (incoming-property-definition): brick.id == brick.source && brick.id != brick.target
    pub(crate) fn add_incoming_property_raw(&self, target: EntityId, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = DataBrick{ id: index, source: index, target, component, data };
        self.add_entity(brick);
        
        index
    }

    /// Creates an entity that is categorized as an outgoing property, one that has its own entity id as a target
    /// An outgoing property `p` of a target `t` looks like this: `p : t -> p`, the property is outgoing from the target
    /// POST-CONDITION (outgoing-property-definition): brick.id == brick.target && brick.id != brick.source
    pub(crate) fn add_outgoing_property_raw(&self, source: EntityId, component: ComponentName, data: Vec<u8>) -> EntityId {
        let index = self.get_next_entity_id();
        let brick = DataBrick{ id: index, source, target: index, component, data };
        self.add_entity(brick);
        
        index
    }

    /// Creates an entity that is categorized as an outgoing property, one that has its own entity id as a target
    /// An outgoing property `p` of a target `t` looks like this: `p : t -> p`, the property is outgoing from the target
    /// POST-CONDITION (outgoing-property-definition): brick.id == brick.target && brick.id != brick.source
    pub(crate) fn add_outgoing_property(&self, source: EntityId, component: ComponentName, fields: Vec<Value>) -> Result<EntityId, String> {
        let matching = self.unify_fields_and_values_into_data(component, fields)
        .map_err(|(cf, d)| 
            format!("[Error][engine_state.rs][add_outgoing_property] Cannot unify field {} (type {:?}) with value {:?} while creating outgoing property {} -> X",
                cf.name, cf.datatype, d, source))?;
    
        let data = matching.concat();
        Ok(self.add_outgoing_property_raw(source, component, data))
    }

    /// Deletes a property entity
    pub(crate) fn delete_property(&self, id: EntityId) {
        self.remove_entity(id);
    }
}

impl Lifecycle for Arc<EngineState> {
    type Entity = EntityId;

    fn create_object(&self, component: ComponentName, fields: Vec<Value>) -> Result<EntityId, String> {
        let matching = self.unify_fields_and_values_into_data(component, fields)
            .map_err(|(cf, d)| 
                format!("[Error][engine_state.rs][create_object] Cannot unify field {} (type {:?}) with value {:?} while creating object",
                    cf.name, cf.datatype, d))?;
        
        let data = matching.concat();
        Ok(self.create_object_raw(component, data))
    }

    fn create_arrow(&self, source: &EntityId, target: &EntityId, component: ComponentName, fields: Vec<Value>) -> Result<EntityId, String> {
        let matching = self.unify_fields_and_values_into_data(component, fields)
            .map_err(|(cf, d)| 
                format!("[Error][engine_state.rs][create_arrow] Cannot unify field {} (type {:?}) with value {:?} while creating arrow {} -> {}",
                    cf.name, cf.datatype, d, source, target))?;
        
        let data = matching.concat();
        Ok(self.create_arrow_raw(*source, *target, component, data))
    }

    fn add_descriptor(&self, target: &EntityId, component: ComponentName, fields: Vec<Value>) -> Result<EntityId, String> {
        let matching = self.unify_fields_and_values_into_data(component, fields)
        .map_err(|(cf, d)| 
            format!("[Error][engine_state.rs][add_incoming_property] Cannot unify field {} (type {:?}) with value {:?} while creating incoming property X -> {}",
                cf.name, cf.datatype, d, target))?;
    
        let data = matching.concat();
        Ok(self.add_incoming_property_raw(*target, component, data))
    }

    fn add_extension(&self, source: &EntityId, component: ComponentName, fields: Vec<Value>) -> Result<EntityId, String> {
        let matching = self.unify_fields_and_values_into_data(component, fields)
        .map_err(|(cf, d)| 
            format!("[Error][engine_state.rs][add_incoming_property] Cannot unify field {} (type {:?}) with value {:?} while creating incoming property X -> {}",
                cf.name, cf.datatype, d, source))?;
    
        let data = matching.concat();
        Ok(self.add_outgoing_property_raw(*source, component, data))
    }
}
/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod engine_state_testing {
    use crate::internals::{datatypes::Datatype, ComponentField, DataBrick, lifecycle::Lifecycle};

    use super::{ComponentType, EngineState};

    #[test]
    fn test_get_next_entity_id() {
        let engine_state = EngineState::new();
        assert_eq!(1, engine_state.get_next_entity_id());
        assert_eq!(2, engine_state.get_next_entity_id());
        assert_eq!(3, engine_state.get_next_entity_id());
    }

    #[test]
    fn test_flatten_simple_alias_component() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias ( ComponentField { name: "Bar".into(), datatype: Datatype::VOID }));
        let foo = ComponentType::Alias ( ComponentField { name: "Foo".into(), datatype: Datatype::COMP("Bar".into()) });
        let flat_foo = engine_state.flatten_component_type(foo);
        assert!(flat_foo.is_ok());
        assert_eq!(Ok(ComponentType::Alias( ComponentField { name: "Foo".into(), datatype: Datatype::VOID })), flat_foo);
    }

    #[test]
    fn test_flatten_complex_alias_component() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Product { name: "Bar".into(), fields: vec![] });
        let foo = ComponentType::Alias ( ComponentField { name: "Foo".into(), datatype: Datatype::COMP("Bar".into()) });
        let flat_foo = engine_state.flatten_component_type(foo);
        assert!(flat_foo.is_ok());
        assert_eq!(Ok(ComponentType::Product { name: "Foo".into(), fields: vec![] }), flat_foo);
    }

    #[test]
    fn test_add_component_type() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias ( ComponentField { name: "Foo".into(), datatype: Datatype::EID }));

        let component_type_index = engine_state.component_type_index.lock().unwrap();

        assert!(!component_type_index.is_empty());
        assert!(component_type_index.contains_key(&"Foo".into()));
        assert!(component_type_index.get(&"Foo".into()).unwrap().is_alias());
        assert_eq!("Foo", component_type_index.get(&"Foo".into()).unwrap().name());
    }

    #[test]
    fn test_get_component_type() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias ( ComponentField { name: "Foo".into(), datatype: Datatype::EID }));

        let output = engine_state.get_component_type("Foo".into());
        assert!(output.is_ok());
        let output = output.unwrap();
        assert!(output.is_alias());
        assert_eq!("Foo", output.name());
    }

    // Testing that insertion creates the needed index storages
    #[test]
    fn test_add_entity() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias ( ComponentField { name: "Foo".into(), datatype: Datatype::EID }));

        let brick = DataBrick{ id: 1, source: 2, target: 3, component: "Foo".into(), data: vec![] };
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
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias ( ComponentField { name: "Foo".into(), datatype: Datatype::EID }));

        let brick = DataBrick{ id: 1, source: 2, target: 3, component: "Foo".into(), data: vec![] };
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
    fn test_create_object_raw() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias (ComponentField { name: "Object".into(), datatype: Datatype::VOID }));

        let object_id = engine_state.create_object_raw("Object".into(), vec![]);
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
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias (ComponentField { name: "Object".into(), datatype: Datatype::VOID }));

        let object_id = engine_state.create_object_raw("Object".into(), vec![]);
        assert!(engine_state.entity_object_index.lock().unwrap().is_member(object_id));
        engine_state.destroy_object(object_id);
        assert!(!engine_state.entity_object_index.lock().unwrap().is_member(object_id));
    }

    #[test]
    fn test_create_arrow_raw() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias (ComponentField { name: "Object".into(), datatype: Datatype::VOID }));
        engine_state.add_raw_component_type(ComponentType::Alias (ComponentField { name: "Arrow".into(), datatype: Datatype::VOID }));

        let one_id = engine_state.create_object_raw("Object".into(), vec![]);
        let two_id = engine_state.create_object_raw("Object".into(), vec![]);
        let three_id = engine_state.create_arrow_raw(one_id, two_id, "Arrow".into(), vec![]);
        let brick_storage = engine_state.entity_brick_storage.lock().unwrap();

        assert!(brick_storage.contains_key(&three_id));
        if let Some(stored_object) = brick_storage.get(&three_id) {
            assert_eq!(three_id, stored_object.id);
            assert_eq!(one_id, stored_object.source);
            assert_eq!(two_id, stored_object.target);
        }
    }

    #[test]
    fn test_high_level_create() {
        let engine_state = EngineState::new();
        engine_state.add_raw_component_type(ComponentType::Alias (ComponentField { name: "Object".into(), datatype: Datatype::VOID }));
        engine_state.add_raw_component_type(ComponentType::Alias (ComponentField { name: "Arrow".into(), datatype: Datatype::VOID }));

        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let ab = engine_state.create_arrow(&a, &b, "Arrow".into(), vec![]).unwrap();

        assert!(engine_state.entity_exists(a));
        assert!(engine_state.entity_exists(b));
        assert!(engine_state.entity_exists(ab));
    }
}