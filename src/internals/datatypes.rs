use std::fmt::Display;

use fstr::FStr;

use super::{byte_utilities::Bytesize, engine_state::EngineState};

/// Entity identifiers are simple usize indices
pub type EntityId = usize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
/// A type representing bound 32-byte strings
pub struct S32(pub FStr<32>);
impl Copy for S32 {}

impl Into<S32> for &str {
    fn into(self) -> S32 {
        S32(FStr::<32>::from_str_lossy(self, b'\0'))
    }
}

impl Into<S32> for &[u8] {
    fn into(self) -> S32 {
        S32(FStr::<32>::from_str_lossy(
            std::str::from_utf8(self).unwrap(),
            b'\0',
        ))
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

impl std::fmt::Debug for S32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.replace("\0", "").trim())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// A type representing unbound, interned strings
pub struct Str(pub u64);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
/// An enumeration of all basic datatypes used in components.
pub enum Datatype {
    /// A void type of size 0 used as markers and tags
    VOID,
    /// Entity ID - equal to usize but will be affected by frame transitions
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
    B256,
    /// A component name and layout - allows for composition
    COMP(S32),
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// A named component field, holding just a name and the field's datatype
pub struct ComponentField {
    pub name: S32,
    pub datatype: Datatype,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// A component type, used for figuring out memory access
pub enum ComponentType {
    /// An alias of a simple datatype
    Alias(ComponentField),
    /// A sum type - an enumeration of types of which only one is selected at a time
    Sum {
        name: S32,
        fields: Vec<ComponentField>,
    },
    /// A product type - a vector of named fields holding different types
    Product {
        name: S32,
        fields: Vec<ComponentField>,
    },
}

impl ComponentType {
    pub fn is_alias(&self) -> bool {
        matches!(self, ComponentType::Alias(_))
    }
    pub fn is_sum(&self) -> bool {
        matches!(self, ComponentType::Sum { .. })
    }
    pub fn is_product(&self) -> bool {
        matches!(self, ComponentType::Product { .. })
    }

    pub fn duplicate_as(&self, new_name: S32) -> ComponentType {
        match self {
            ComponentType::Alias(ComponentField { name: _, datatype }) => {
                ComponentType::Alias(ComponentField {
                    name: new_name,
                    datatype: datatype.clone(),
                })
            }
            ComponentType::Sum { name: _, fields } => ComponentType::Sum {
                name: new_name,
                fields: fields.clone(),
            },
            ComponentType::Product { name: _, fields } => ComponentType::Product {
                name: new_name,
                fields: fields.clone(),
            },
        }
    }

    pub fn name(&self) -> String {
        let s = match self {
            ComponentType::Alias(ComponentField { name, .. }) => name.0.to_string(),
            ComponentType::Sum { name, .. } => name.0.to_string(),
            ComponentType::Product { name, .. } => name.0.to_string(),
        };

        s.replace("\0", "")
    }

    pub fn get_field_names(&self) -> Vec<S32> {
        self.get_fields()
            .iter()
            .map(|comp| comp.name.clone())
            .collect()
    }

    /// Returns the fields of a certain component types
    pub fn get_fields(&self) -> Vec<ComponentField> {
        match self {
            ComponentType::Alias(field) => vec![field.clone()],
            ComponentType::Sum { fields, .. } => fields.clone(),
            ComponentType::Product { fields, .. } => fields.clone(),
        }
    }

    pub fn get_field(&self, field_name: S32) -> Option<&ComponentField> {
        match self {
            ComponentType::Alias(field) if field.name == "self".into() => Some(field),
            ComponentType::Sum { fields, .. } => fields.iter().find(|f| f.name == field_name),
            ComponentType::Product { fields, .. } => fields.iter().find(|f| f.name == field_name),
            _ => None,
        }
    }
}

pub fn try_read_component_type(
    engine: &EngineState,
    input: &[u8],
) -> Result<ComponentType, String> {
    let component_name_length = 32;
    let input_length = input.len();

    if input_length < component_name_length {
        return Err(format!("[Error][datatypes.rs][try_read_component_type] Input not long enough to read type name."));
    }

    let message_length = input.len() - component_name_length;

    let message_type = &input[0..component_name_length];
    let utf8_name = std::str::from_utf8(message_type).map_err(|e| e.to_string())?;
    let component_name: S32 = utf8_name.into();

    let component_type = engine.get_component_type(component_name)?;
    let bytesize = component_type.bytesize(engine);
    if 8 * bytesize != message_length {
        return Err(format!("[Error][datatypes.rs][try_read_component_type] Expected message length for type '{}' is {} bytes, but {} bytes found in input: {:?}.",
            component_name, bytesize, message_length, input));
    }

    return Ok(component_type);
}

pub type B256 = fstr::FStr<256>;

#[derive(Debug, PartialEq, Clone)]
/// A datatype value that holds the type and value for some variable
pub enum Value {
    /// A void type of size 0 used as markers and tags
    VOID,
    /// Entity ID - equal to U32 but will be affected by frame transitions
    EID(EntityId),
    /// A 64-bit signed integer number
    I32(i32),
    /// A 64-bit signed integer number
    I64(i64),
    /// A 32-bit unsigned integer number
    U32(u32),
    /// A 64-bit unsigned integer number
    U64(u64),
    /// A 32-bit floating-point number
    F32(f32),
    /// A 64-bit floating-point number
    F64(f64),
    /// A 32-bit bound-size string
    S32(S32),
    /// An interned unbound string
    B256(B256),
}

impl Value {
    /// Gets the datatype from the datatype and value pair
    pub fn get_datatype(&self) -> Datatype {
        match self {
            Value::VOID => Datatype::VOID,
            Value::EID(_) => Datatype::EID,
            Value::I32(_) => Datatype::I32,
            Value::I64(_) => Datatype::I64,
            Value::U32(_) => Datatype::U32,
            Value::U64(_) => Datatype::U64,
            Value::F32(_) => Datatype::F32,
            Value::F64(_) => Datatype::F64,
            Value::S32(_) => Datatype::S32,
            Value::B256(_) => Datatype::B256,
        }
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod datatypes_testing {
    use crate::internals::{
        datatypes::ComponentField, engine_state::EngineState, lifecycle::Lifecycle,
    };

    use super::{try_read_component_type, ComponentType, Datatype, Value, S32};

    #[test]
    fn test_try_read_alias() {
        let engine_state = EngineState::new();
        let component_type = ComponentType::Alias({
            ComponentField {
                name: "foo".into(),
                datatype: Datatype::EID,
            }
        });
        engine_state.add_raw_component_type(component_type.clone());

        let input = {
            let mut buffer: Vec<u8> = vec![];
            let name: S32 = "foo".into();
            buffer.extend(name.0.as_bytes());
            buffer.extend(vec![0u8; 64]);
            buffer
        };

        assert_eq!(
            Ok(component_type),
            try_read_component_type(&engine_state, input.as_slice())
        );
    }

    #[test]
    fn test_try_read_product() {
        let engine_state = EngineState::new();
        let component_type = ComponentType::Product {
            name: "foo".into(),
            fields: vec![
                ComponentField {
                    name: "a".into(),
                    datatype: Datatype::EID,
                },
                ComponentField {
                    name: "b".into(),
                    datatype: Datatype::U32,
                },
            ],
        };

        engine_state.add_raw_component_type(component_type.clone());

        let input = {
            let mut buffer: Vec<u8> = vec![];
            let name: S32 = "foo".into();
            buffer.extend(name.0.as_bytes());
            buffer.extend(vec![0u8; 64]);
            buffer.extend(vec![0u8; 32]);
            buffer
        };

        assert_eq!(
            Ok(component_type),
            try_read_component_type(&engine_state, input.as_slice())
        );
    }

    #[test]
    fn test_large_numbers() {
        let engine_state = EngineState::new();
        engine_state.add_component_types("Num : u32;").unwrap();

        let a = engine_state
            .create_object("Num".into(), vec![Value::U32(4294967294)])
            .unwrap();

        let brick = engine_state.get_brick(a).unwrap();
        assert_eq!(vec![255u8, 255u8, 255u8, 254u8], brick.data);
    }
}
