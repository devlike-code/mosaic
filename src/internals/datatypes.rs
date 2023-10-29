use std::fmt::Display;

use fstr::FStr;

use super::{byte_utilities::Bytesize, engine_state::EngineState};

/// Entity identifiers are simple usize indices
pub type EntityId = usize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Debug)]
/// A type representing bound 32-byte strings
pub struct S32(pub FStr<32>);
impl Copy for S32 {}

impl Into<S32> for &str {
    fn into(self) -> S32 {
        S32(FStr::<32>::from_str_lossy(self, b'\0'))
    }
}

impl Into<S32> for String {
    fn into(self) -> S32 {
        S32(FStr::<32>::from_str_lossy(self.as_str(), b'\0'))
    }
}

impl Display for S32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.replace("\0", "").trim())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Debug)]
/// A type representing unbound, interned strings
pub struct Str(pub u64);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
#[derive(Debug)]
/// An enumeration of all basic datatypes used in components.
pub enum Datatype {
    /// A void type of size 0 used as markers and tags
    VOID,
    /// Entity ID - equal to U32 but will be affected by frame transitions
    EID,
    /// A 64-bit signed integer number
    I32,
    /// A 64-bit signed integer number
    I64,
    /// A 32-bit unsigned integer number
    U32,
    /// A 64-bit unsigned integer number
    U64,
    /// A 32-bit floating-point number
    F32,
    /// A 64-bit floating-point number
    F64,
    /// A 32-bit bound-size string
    S32,
    /// An interned unbound string
    STR,
    /// A component name and layout - allows for composition
    COMP(S32)
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Debug)]
/// A named component field, holding just a name and the field's datatype
pub struct ComponentField {
    pub name: S32,
    pub datatype: Datatype,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Debug)]
/// A component type, used for figuring out memory access
pub enum ComponentType {
    /// An alias of a simple datatype
    Alias { name: S32, aliased: Datatype },
    /// A sum type - an enumeration of types of which only one is selected at a time
    Sum { name: S32, fields: Vec<ComponentField> },
    /// A product type - a vector of named fields holding different types
    Product { name: S32, fields: Vec<ComponentField> },
}

impl ComponentType {
    pub fn is_alias(&self) -> bool { matches!(self, ComponentType::Alias { .. }) }
    pub fn is_sum(&self) -> bool { matches!(self, ComponentType::Sum { .. }) }
    pub fn is_product(&self) -> bool { matches!(self, ComponentType::Product { .. }) }

    pub fn name(&self) -> String {
        let s = match self {
            ComponentType::Alias { name, .. } => name.0.to_string(),
            ComponentType::Sum{ name, .. } => name.0.to_string(),
            ComponentType::Product{ name, .. } => name.0.to_string(),
        };

        s.replace("\0", "")
    }

    pub fn get_field_names(&self) -> Vec<S32> {
        self.get_fields().iter().map(|comp| comp.name.clone()).collect()
    }

    /// Returns the fields of a certain component types
    pub fn get_fields(&self) -> &[ComponentField] {
        match self {
            ComponentType::Alias{ .. } => &[],
            ComponentType::Sum{ fields, .. } => fields,
            ComponentType::Product{ fields, .. } => fields,
        }
    }
}

pub fn try_read_component_type(engine: &EngineState, input: &[u8]) -> Result<ComponentType, String> {
    let component_name_length = 32;
    let input_length = input.len();
    
    if input_length < component_name_length {
        return Err(format!("[Error][datatypes.rs][try_read_component_type] Input not long enough to read type name."))
    }

    let message_length = input.len() - component_name_length;

    let message_type = &input[0..component_name_length];
    let utf8_name = std::str::from_utf8(message_type)
        .map_err(|e| e.to_string())?;
    let component_name: S32 = utf8_name.into();

    if let Some(component_type) = engine.get_component_type(component_name) {
        let bytesize = component_type.bytesize(engine);
        if 8 * bytesize != message_length {
            return Err(format!("[Error][datatypes.rs][try_read_component_type] Expected message length for type '{}' is {} bytes, but {} bytes found in input: {:?}.",
                component_name, bytesize, message_length, input));
        }

        return Ok(component_type)
    } else {
        return Err(format!("[Error][datatypes.rs][try_read_component_type] Component '{}' not found in component type index.", component_name));
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod datatypes_testing {
    use crate::internals::{engine_state::EngineState, datatypes::ComponentField};

    use super::{ComponentType, S32, Datatype, try_read_component_type};

    #[test]
    fn test_component_type_try_read_alias() {
        let engine_state = EngineState::default();
        let component_type = ComponentType::Alias { name: "foo".into(), aliased: Datatype::EID };
        engine_state.add_component_type(component_type.clone());
        
        let input = {
            let mut buffer: Vec<u8> = vec![];
            let name: S32 = "foo".into();
            buffer.extend(name.0.as_bytes());
            buffer.extend(vec![0u8; 64]);
            buffer
        };

        assert_eq!(Ok(component_type), try_read_component_type(&engine_state, input.as_slice()));        
    }

    #[test]
    fn test_component_type_try_read_product() {
        let engine_state = EngineState::default();
        let component_type = ComponentType::Product { name: "foo".into(),
            fields: vec![
                ComponentField { name: "a".into(), datatype: Datatype::EID },
                ComponentField { name: "b".into(), datatype: Datatype::U32 },
            ] 
        };

        engine_state.add_component_type(component_type.clone());
        
        let input = {
            let mut buffer: Vec<u8> = vec![];
            let name: S32 = "foo".into();
            buffer.extend(name.0.as_bytes());
            buffer.extend(vec![0u8; 64]);
            buffer.extend(vec![0u8; 32]);
            buffer
        };

        assert_eq!(Ok(component_type), try_read_component_type(&engine_state, input.as_slice()));
    }
}