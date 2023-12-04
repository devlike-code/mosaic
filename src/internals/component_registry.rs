use itertools::Itertools;

use super::{
    component_grammar::ComponentParser,
    datatypes::{ComponentType, S32 as ComponentName},
    logging::Logging,
    Bytesize, ComponentField, Datatype, ToByteArray, Value,
};

use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, Mutex},
};

type FieldName = ComponentName;

#[derive(Default, Debug)]
pub struct ComponentRegistry {
    pub component_type_map: Mutex<HashMap<ComponentName, ComponentType>>,
    pub component_offset_size_map: Mutex<HashMap<(String, FieldName), Range<usize>>>,
    pub component_definitions: Mutex<Vec<String>>,
}

impl PartialEq for ComponentRegistry {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for ComponentRegistry {}

impl ComponentRegistry {
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

    fn add_raw_component_type(&self, definition: ComponentType) -> ComponentType {
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

        definition
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
                    datatype: Datatype::UNIT,
                },
                Value::UNIT,
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

impl ComponentRegistry {
    pub fn new() -> Arc<ComponentRegistry> {
        Arc::new(ComponentRegistry::default())
    }

    pub fn add_component_types(&self, definition: &str) -> anyhow::Result<Vec<ComponentType>> {
        let types = ComponentParser::parse_all(definition)?
            .into_iter()
            .flat_map(|t| self.flatten_component_type(t))
            .map(|t| self.add_raw_component_type(t))
            .collect_vec();

        self.component_definitions
            .lock()
            .unwrap()
            .push(definition.to_owned());

        Ok(types)
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
