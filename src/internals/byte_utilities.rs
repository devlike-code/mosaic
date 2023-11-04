use fstr::FStr;
use std::convert::AsMut;

use super::{DatatypeValue, B256};
use super::datatypes::{ComponentField, ComponentType, Datatype, Str, S32};
use super::engine_state::EngineState;

/// A trait that makes it very clear what the bytesize of a particular struct is meant to be, when statically known
pub(crate) trait Bytesize {
    fn bytesize(self: &Self, engine: &EngineState) -> usize;
}

/// Representation for anything that can be deserialized from a byte array
pub trait FromByteArray {
    fn from_byte_array(data: &[u8]) -> Self;
}

/// Representation for anything that can be serialized into a byte array
pub trait ToByteArray {
    fn to_byte_array(&self) -> Vec<u8>;
}

/// A useful helper function for copying bytes
fn copy_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

/// The `FromByteArray` implementation for `u32`
impl FromByteArray for u32 {
    fn from_byte_array(data: &[u8]) -> Self {
        u32::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `u32`
impl ToByteArray for u32 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `i32`
impl FromByteArray for i32 {
    fn from_byte_array(data: &[u8]) -> Self {
        i32::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `i32`
impl ToByteArray for i32 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `usize`
impl FromByteArray for usize {
    fn from_byte_array(data: &[u8]) -> Self {
        usize::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `usize`
impl ToByteArray for usize {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `u64`
impl FromByteArray for u64 {
    fn from_byte_array(data: &[u8]) -> Self {
        u64::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `u64`
impl ToByteArray for u64 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `i64`
impl FromByteArray for i64 {
    fn from_byte_array(data: &[u8]) -> Self {
        i64::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `i64`
impl ToByteArray for i64 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `f32`
impl FromByteArray for f32 {
    fn from_byte_array(data: &[u8]) -> Self {
        f32::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `f32`
impl ToByteArray for f32 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `f64`
impl FromByteArray for f64 {
    fn from_byte_array(data: &[u8]) -> Self {
        f64::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `f64`
impl ToByteArray for f64 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `u8`
impl FromByteArray for u8 {
    fn from_byte_array(data: &[u8]) -> Self {
        data[0]
    }
}

/// The `ToByteArray` implementation for `u8`
impl ToByteArray for u8 {
    fn to_byte_array(&self) -> Vec<u8> {
        vec![*self]
    }
}

/// The `FromByteArray` implementation for `s32`
impl FromByteArray for S32 {
    fn from_byte_array(data: &[u8]) -> Self {
        let str = std::str::from_utf8(data);
        S32(FStr::<32>::from_str_lossy(str.unwrap(), b'\0'))
    }
}

/// The `ToByteArray` implementation for `s32`
impl ToByteArray for S32 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `str`
impl FromByteArray for Str {
    fn from_byte_array(data: &[u8]) -> Self {
        Str(u64::from_byte_array(data))
    }
}

/// The `ToByteArray` implementation for `s32`
impl ToByteArray for B256 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `str`
impl FromByteArray for B256 {
    fn from_byte_array(data: &[u8]) -> Self {
        let str = std::str::from_utf8(data);
        FStr::<256>::from_str_lossy(str.unwrap(), b'\0')
    }
}

/// The `ToByteArray` implementation for `str`
impl ToByteArray for Str {
    fn to_byte_array(&self) -> Vec<u8> {
        self.0.to_byte_array()
    }
}

/// A bytesize check for complex component datatypes
impl Bytesize for ComponentType {
    fn bytesize(self: &Self, engine: &EngineState) -> usize {
        match self {
            ComponentType::Alias(field) => field.datatype.bytesize(engine),
            ComponentType::Sum { fields, .. } => fields
                .iter()
                .fold(0usize, |old, ComponentField { datatype, .. }| {
                    old + datatype.bytesize(engine)
                }),
            ComponentType::Product { fields, .. } => fields
                .iter()
                .fold(0usize, |old, ComponentField { datatype, .. }| {
                    old + datatype.bytesize(engine)
                }),
        }
    }
}

/// A bytesize check for all basic component datatypes
impl Bytesize for Datatype {
    fn bytesize(self: &Self, engine: &EngineState) -> usize {
        match self {
            Datatype::VOID => 0usize,
            Datatype::I32 | Datatype::U32 | Datatype::F32 | Datatype::S32 => 4usize,
            Datatype::I64 | Datatype::U64 | Datatype::F64 | Datatype::EID => 8usize,
            Datatype::B256 => 32usize,
            Datatype::COMP(component_name) => engine
                .get_component_type(*component_name)
                .map(|t| t.bytesize(engine))
                .unwrap_or(0usize),
        }
    }
}

pub fn slice_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

impl ToByteArray for DatatypeValue {
    fn to_byte_array(&self) -> Vec<u8> {
        match self {
            DatatypeValue::VOID => vec![],
            DatatypeValue::EID(eid) => (*eid).to_byte_array(),
            DatatypeValue::I32(i) => (*i).to_byte_array(),
            DatatypeValue::I64(i) => (*i).to_byte_array(),
            DatatypeValue::U32(u) => (*u).to_byte_array(),
            DatatypeValue::U64(u) => (*u).to_byte_array(),
            DatatypeValue::F32(f) => (*f).to_byte_array(),
            DatatypeValue::F64(f) => (*f).to_byte_array(),
            DatatypeValue::S32(s) => s.to_byte_array(),
            DatatypeValue::B256(b) => b.to_byte_array(),
        }
    }
}