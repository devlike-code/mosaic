use std::fmt::Display;

use fstr::FStr;

use super::{logging::Logging, Bytesize, ComponentRegistry};

pub type EntityId = usize;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct S32(pub FStr<32>);
impl Copy for S32 {}

impl S32 {
    pub fn to_string(&self) -> String {
        self.0.replace('\0', "").trim().into()
    }
}

impl From<&str> for S32 {
    fn from(value: &str) -> Self {
        S32(FStr::<32>::from_str_lossy(value, b'\0'))
    }
}

impl From<&[u8]> for S32 {
    fn from(value: &[u8]) -> Self {
        S32(FStr::<32>::from_str_lossy(
            std::str::from_utf8(value).unwrap(),
            b'\0',
        ))
    }
}

impl From<String> for S32 {
    fn from(value: String) -> Self {
        S32(FStr::<32>::from_str_lossy(value.as_str(), b'\0'))
    }
}

impl Display for S32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.replace('\0', "").trim())
    }
}

impl std::fmt::Debug for S32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.replace('\0', "").trim())
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Str(pub u64);

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub enum Datatype {
    UNIT,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    S32,
    S128,
    BOOL,
    COMP(S32),
}

pub fn self_val(value: Value) -> Vec<(S32, Value)> {
    vec![("self".into(), value)]
}

pub fn default_vals() -> Vec<(S32, Value)> {
    vec![]
}

impl Datatype {
    pub fn get_default(&self) -> Value {
        match self {
            Datatype::UNIT => Value::UNIT(()),
            // COMP fields will disappear when the component is added to the engine state,
            // so this situation should never arise. However, we'll leave a log here just in case.
            Datatype::COMP(_) => Value::UNIT(()),
            Datatype::I8 => Value::I8(0),
            Datatype::I16 => Value::I16(0),
            Datatype::I32 => Value::I32(0),
            Datatype::I64 => Value::I64(0),
            Datatype::U8 => Value::U8(0),
            Datatype::U16 => Value::U16(0),
            Datatype::U32 => Value::U32(0),
            Datatype::U64 => Value::U64(0),
            Datatype::F32 => Value::F32(0.0),
            Datatype::F64 => Value::F64(0.0),
            Datatype::S32 => Value::S32("".into()),
            Datatype::S128 => Value::S128(vec![]),
            Datatype::BOOL => Value::BOOL(false),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ComponentField {
    pub name: S32,
    pub datatype: Datatype,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum ComponentType {
    Alias(ComponentField),

    Product {
        name: S32,
        fields: Vec<ComponentField>,
    },
}

impl ComponentType {
    pub fn is_alias(&self) -> bool {
        matches!(self, ComponentType::Alias(_))
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
            ComponentType::Product { name: _, fields } => ComponentType::Product {
                name: new_name,
                fields: fields.clone(),
            },
        }
    }

    pub fn name(&self) -> String {
        let s = match self {
            ComponentType::Alias(ComponentField { name, .. }) => name.0.to_string(),
            ComponentType::Product { name, .. } => name.0.to_string(),
        };

        s.replace('\0', "")
    }

    pub fn get_field_names(&self) -> Vec<S32> {
        self.get_fields().iter().map(|comp| comp.name).collect()
    }

    pub fn get_fields(&self) -> Vec<ComponentField> {
        match self {
            ComponentType::Alias(field) => vec![field.clone()],
            ComponentType::Product { fields, .. } => fields.clone(),
        }
    }

    pub fn get_field(&self, field_name: S32) -> Option<&ComponentField> {
        match self {
            ComponentType::Alias(field) if field.name == "self".into() => Some(field),
            ComponentType::Product { fields, .. } => fields.iter().find(|f| f.name == field_name),
            _ => None,
        }
    }
}

pub fn try_read_component_type(
    engine: &ComponentRegistry,
    input: &[u8],
) -> anyhow::Result<ComponentType> {
    let component_name_length = 32;
    let input_length = input.len();

    if input_length < component_name_length {
        return "Input not long enough to read type name.".to_error();
    }

    let message_length = input.len() - component_name_length;

    let message_type = &input[0..component_name_length];
    let utf8_name = std::str::from_utf8(message_type)?;
    let component_name: S32 = utf8_name.into();

    let component_type = engine.get_component_type(component_name)?;
    let bytesize = component_type.bytesize(engine);
    if 8 * bytesize != message_length {
        format!(
            "Expected message length for type '{}' is {} bytes, but {} bytes found in input: {:?}.",
            component_name, bytesize, message_length, input
        )
        .to_error()
    } else {
        Ok(component_type)
    }
}

pub type S128 = Vec<u8>;

pub type ComponentValues = Vec<(S32, Value)>;

#[derive(Debug, PartialEq, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Value {
    UNIT(()),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    S32(S32),
    S128(S128),
    BOOL(bool),
}

impl Value {
    pub fn get_datatype(&self) -> Datatype {
        match self {
            Value::UNIT(()) => Datatype::UNIT,
            Value::I8(_) => Datatype::I8,
            Value::I16(_) => Datatype::I16,
            Value::I32(_) => Datatype::I32,
            Value::I64(_) => Datatype::I64,
            Value::U8(_) => Datatype::U8,
            Value::U16(_) => Datatype::U16,
            Value::U32(_) => Datatype::U32,
            Value::U64(_) => Datatype::U64,
            Value::F32(_) => Datatype::F32,
            Value::F64(_) => Datatype::F64,
            Value::S32(_) => Datatype::S32,
            Value::S128(_) => Datatype::S128,
            Value::BOOL(_) => Datatype::BOOL,
        }
    }

    pub fn as_i8(&self) -> i8 {
        match self {
            Value::I8(v) => *v,
            _ => panic!("Cannot get type variant I8"),
        }
    }

    pub fn as_i16(&self) -> i16 {
        match self {
            Value::I16(v) => *v,
            _ => panic!("Cannot get type variant I16"),
        }
    }

    pub fn as_i32(&self) -> i32 {
        match self {
            Value::I32(v) => *v,
            _ => panic!("Cannot get type variant I32"),
        }
    }

    pub fn as_i64(&self) -> i64 {
        match self {
            Value::I64(v) => *v,
            _ => panic!("Cannot get type variant I64"),
        }
    }

    pub fn as_u8(&self) -> u8 {
        match self {
            Value::U8(v) => *v,
            _ => panic!("Cannot get type variant U8"),
        }
    }

    pub fn as_u16(&self) -> u16 {
        match self {
            Value::U16(v) => *v,
            _ => panic!("Cannot get type variant U16"),
        }
    }

    pub fn as_u32(&self) -> u32 {
        match self {
            Value::U32(v) => *v,
            _ => panic!("Cannot get type variant U32"),
        }
    }

    pub fn as_u64(&self) -> u64 {
        match self {
            Value::U64(v) => *v,
            _ => panic!("Cannot get type variant U64"),
        }
    }

    pub fn as_f32(&self) -> f32 {
        match self {
            Value::F32(v) => *v,
            _ => panic!("Cannot get type variant F32"),
        }
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Value::F64(v) => *v,
            _ => panic!("Cannot get type variant F64"),
        }
    }

    pub fn as_s32(&self) -> S32 {
        match self {
            Value::S32(v) => *v,
            _ => panic!("Cannot get type variant S32"),
        }
    }

    pub fn as_s128(&self) -> S128 {
        match self {
            Value::S128(v) => v.clone(),
            _ => panic!("Cannot get type variant s128"),
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::BOOL(v) => v.clone(),
            _ => panic!("Cannot get type variant bool"),
        }
    }
}
