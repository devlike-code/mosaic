use slab::Slab;

use super::{
    component_grammar::ComponentParser,
    datatypes::{ComponentType, EntityId, S32 as ComponentName},
    logging::Logging,
    Bytesize, ComponentField, Datatype, ToByteArray, Value,
};

use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, Mutex},
};

type FieldName = ComponentName;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub(crate) struct DataBrick {
    // Structural data (24 bytes)
    pub(crate) id: EntityId,
    pub(crate) source: EntityId,
    pub(crate) target: EntityId,
    // Component (32 bytes)
    pub(crate) component: ComponentName,
    // Data (fills to 256 bytes)
    pub(crate) data: [u8; 200],
}

impl DataBrick {
    pub(crate) fn new(
        id: EntityId,
        source: EntityId,
        target: EntityId,
        component: ComponentName,
    ) -> DataBrick {
        DataBrick {
            id,
            source,
            target,
            component,
            data: [0; 200],
        }
    }
}

#[derive(Default, Debug)]
pub struct EntityRegistry {
    pub component_type_map: Mutex<HashMap<ComponentName, ComponentType>>,
    pub component_offset_size_map: Mutex<HashMap<(String, FieldName), Range<usize>>>,
    pub id_allocation_index: Mutex<HashMap<EntityId, usize>>,
    pub(crate) component_slabs: Mutex<HashMap<ComponentName, Slab<DataBrick>>>,
}

impl PartialEq for EntityRegistry {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for EntityRegistry {}

impl EntityRegistry {
    fn flatten_component_type(&self, definition: ComponentType) -> anyhow::Result<ComponentType> {
        use ComponentType::*;
        match &definition {
            Alias(ComponentField {
                name: _,
                datatype: Datatype::COMP(other),
            }) => {
                let other_type = self.get_component_type(*other)?;
                Ok(other_type.duplicate_as(definition.name().into()))
            }
            _ => Ok(definition),
        }
    }

    fn add_raw_component_type(&self, definition: ComponentType) {
        self.component_type_map
            .lock()
            .unwrap()
            .insert(definition.name().into(), definition.clone());

        let mut offset_size_index = self.component_offset_size_map.lock().unwrap();

        let mut offset = 0usize;
        for field in definition.get_fields() {
            let size = field.datatype.bytesize(self);
            let range = offset..offset + size;
            offset_size_index.insert((definition.name().to_string(), field.name), range);
            offset += size;
        }

        self.component_slabs
            .lock()
            .unwrap()
            .insert(definition.name().into(), Slab::new());
    }

    fn unify_fields_and_values_into_data(
        &self,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Vec<Vec<u8>>, Box<(ComponentField, Value)>> {
        let components = self.component_type_map.lock().unwrap();
        let component_type = components
            .get(&component)
            .ok_or((
                ComponentField {
                    name: format!("<{}>", component).as_str().into(),
                    datatype: Datatype::VOID,
                },
                Value::VOID,
            ))?
            .clone();
        let mut has_error = None;
        let fields = component_type
            .get_fields()
            .into_iter()
            .zip(fields)
            .map(|(field, datatype_value)| {
                if datatype_value.get_datatype() == field.datatype {
                    Ok(datatype_value.to_byte_array())
                } else {
                    has_error = Some((field.clone(), datatype_value.clone()));
                    Err((field, datatype_value))
                }
            })
            .collect::<Vec<_>>();

        if let Some(error) = has_error {
            Err(Box::new(error))
        } else {
            Ok(fields.iter().map(|e| e.clone().unwrap()).collect())
        }
    }
}

impl EntityRegistry {
    pub fn new() -> Arc<EntityRegistry> {
        Arc::new(EntityRegistry::default())
    }

    pub fn add_component_types(&self, definition: &str) -> anyhow::Result<()> {
        let types = ComponentParser::parse_all(definition)?;
        for component_type in types {
            self.add_raw_component_type(self.flatten_component_type(component_type)?);
        }
        Ok(())
    }

    pub fn has_component_type(&self, name: &ComponentName) -> bool {
        self.component_type_map.lock().unwrap().contains_key(name)
    }

    pub fn get_component_type(&self, name: ComponentName) -> anyhow::Result<ComponentType> {
        if self.has_component_type(&name) {
            if let Some(typ) = self.component_type_map.lock().unwrap().get(&name).cloned() {
                Ok(typ)
            } else {
                format!("Component with name {} not found", name).to_error()
            }
        } else {
            format!("Component with name {} not found", name).to_error()
        }
    }
}
