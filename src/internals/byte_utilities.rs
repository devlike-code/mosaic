use fstr::FStr;
use std::convert::AsMut;

use super::component_registry::ComponentRegistry;
use super::datatypes::{ComponentField, ComponentType, Datatype, Str, S32};
use super::{Value, S128};

/// A trait that makes it very clear what the bytesize of a particular struct is meant to be, when statically known
pub(crate) trait Bytesize {
    fn bytesize(&self, engine: &ComponentRegistry) -> usize;
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

/// The `FromByteArray` implementation for `u8`
impl FromByteArray for u8 {
    fn from_byte_array(data: &[u8]) -> Self {
        u8::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `u8`
impl ToByteArray for u8 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `u16`
impl FromByteArray for u16 {
    fn from_byte_array(data: &[u8]) -> Self {
        u16::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `u16`
impl ToByteArray for u16 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
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

/// The `FromByteArray` implementation for `bool`
impl FromByteArray for bool {
    fn from_byte_array(data: &[u8]) -> Self {
        data[0] == 1
    }
}

/// The `ToByteArray` implementation for `bool`
impl ToByteArray for bool {
    fn to_byte_array(&self) -> Vec<u8> {
        vec![if *self { 1 } else { 0 }]
    }
}

/// The `FromByteArray` implementation for `i8`
impl FromByteArray for i8 {
    fn from_byte_array(data: &[u8]) -> Self {
        i8::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `i8`
impl ToByteArray for i8 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_be_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `i16`
impl FromByteArray for i16 {
    fn from_byte_array(data: &[u8]) -> Self {
        i16::from_be_bytes(copy_into_array(data))
    }
}

/// The `ToByteArray` implementation for `i16`
impl ToByteArray for i16 {
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

/// The `FromByteArray` implementation for `s32`
impl FromByteArray for S32 {
    fn from_byte_array(data: &[u8]) -> Self {
        let str = std::str::from_utf8(data);
        S32(FStr::<32>::from_str_lossy(str.unwrap(), b'\0'))
    }
}

/// The `ToByteArray` implementation for `String`
impl ToByteArray for String {
    fn to_byte_array(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

/// The `FromByteArray` implementation for `String`
impl FromByteArray for String {
    fn from_byte_array(data: &[u8]) -> Self {
        let str = std::str::from_utf8(data);
        String::from_utf8_lossy(str.unwrap().as_bytes()).into_owned()
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
impl ToByteArray for S128 {
    fn to_byte_array(&self) -> Vec<u8> {
        self.to_vec()
    }
}

/// The `FromByteArray` implementation for `str`
impl FromByteArray for S128 {
    fn from_byte_array(data: &[u8]) -> Self {
        data.try_into()
            .expect("Cannot turn slice into array and satisfy conditions for s128")
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
    fn bytesize(&self, engine: &ComponentRegistry) -> usize {
        match self {
            ComponentType::Alias(field) => field.datatype.bytesize(engine),
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
    fn bytesize(&self, engine: &ComponentRegistry) -> usize {
        match self {
            Datatype::UNIT => 0usize,
            Datatype::BOOL => 1usize,
            Datatype::I8 | Datatype::U8 => 1usize,
            Datatype::I16 | Datatype::U16 => 2usize,
            Datatype::I32 | Datatype::U32 | Datatype::F32 => 4usize,
            Datatype::I64 | Datatype::U64 | Datatype::F64 => 8usize,
            Datatype::S32 => 32usize,
            Datatype::S128 => 128usize,
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

impl ToByteArray for Value {
    fn to_byte_array(&self) -> Vec<u8> {
        match self {
            Value::UNIT => vec![],
            Value::I8(i) => (*i).to_byte_array(),
            Value::I16(i) => (*i).to_byte_array(),
            Value::I32(i) => (*i).to_byte_array(),
            Value::I64(i) => (*i).to_byte_array(),
            Value::U8(u) => (*u).to_byte_array(),
            Value::U16(u) => (*u).to_byte_array(),
            Value::U32(u) => (*u).to_byte_array(),
            Value::U64(u) => (*u).to_byte_array(),
            Value::F32(f) => (*f).to_byte_array(),
            Value::F64(f) => (*f).to_byte_array(),
            Value::S32(s) => s.to_byte_array(),
            Value::S128(b) => b.to_byte_array(),
            Value::BOOL(b) => b.to_byte_array(),
        }
    }
}
