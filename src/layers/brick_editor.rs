
use fstr::FStr;

use crate::internals::{
    Bytesize, ComponentField, ComponentType, Datatype, EngineState, EntityId, S32,
};

use super::querying::Querying;

fn copy_into_array<A, T>(slice: &[T]) -> A
where
    A: Default + AsMut<[T]>,
    T: Copy,
{
    let mut a = A::default();
    <A as AsMut<[T]>>::as_mut(&mut a).copy_from_slice(slice);
    a
}

#[derive(Debug)]
pub struct BrickEditor<'a> {
    pub(crate) engine: &'a EngineState,
    brick_id: EntityId,
}

#[derive(Debug)]
pub struct FieldEditor<'f, 'e: 'f> {
    brick_editor: &'f BrickEditor<'e>,
    data: DatatypeValue,
}

pub type B256 = fstr::FStr<256>;

#[derive(Debug, PartialEq)]
pub enum DatatypeValue {
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
    /// A component name and layout - allows for composition
    COMP,
   
}

impl<'a> BrickEditor<'a> {
    pub fn get_field_editor(&self, field_name: S32) -> Option<FieldEditor> {
        if let Some(brick) = self.engine.get(self.brick_id) {
            if let Some(component_type) = self.engine.get_component_type(brick.component) {
                if let Some(field) = component_type.get_field(field_name) {
                    if let Some(offset) = self.get_field_offset(&component_type, field_name){
                        let offset_bytesize = field.datatype.bytesize(self.engine) + offset;
                        let field_data_raw = &brick.data[offset..offset_bytesize];
                        let value: DatatypeValue = match field.datatype {
                            Datatype::VOID => DatatypeValue::VOID,
                            Datatype::I32 => DatatypeValue::I32(i32::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::U32 => DatatypeValue::U32(u32::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::F32 => DatatypeValue::F32(f32::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::S32 => DatatypeValue::S32(field_data_raw.into()),
                            Datatype::I64 => DatatypeValue::I64(i64::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::U64 => DatatypeValue::U64(u64::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::F64 => DatatypeValue::F64(f64::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::EID => DatatypeValue::EID(usize::from_ne_bytes(copy_into_array(field_data_raw))),
                            Datatype::B256 => DatatypeValue::B256(FStr::<256>::from_str_lossy(std::str::from_utf8(field_data_raw).unwrap(), b'\0')),
                            Datatype::COMP(component_name) => DatatypeValue::COMP,
                        };
                    
                        return Some(FieldEditor {
                            brick_editor: &self,
                            data: value,
                        });
                    }
                }            
            }
        }
        None
    }

    pub fn set_field(&self, field: ComponentField, field_data: DatatypeValue) -> Result<(), String> {
        if let Some(field_editor) = self.get_field_editor(field.name) {
            if let Some(mut brick) = self.engine.get(self.brick_id) {
                let mut flag = false;
                let value: Vec<u8> = match (field.datatype.clone(), field_data) {
                    (Datatype::VOID, DatatypeValue::VOID) => vec![],
                    (Datatype::I32, DatatypeValue::I32(x)) => x.to_ne_bytes().to_vec(),
                    (Datatype::U32, DatatypeValue::U32(x)) => x.to_ne_bytes().to_vec(),
                    (Datatype::F32, DatatypeValue::F32(x)) => x.to_ne_bytes().to_vec(),
                    (Datatype::S32, DatatypeValue::S32(x)) => x.0.as_bytes().to_vec(),
                    (Datatype::I64, DatatypeValue::I64(x)) =>x.to_ne_bytes().to_vec(),
                    (Datatype::U64, DatatypeValue::U64(x)) =>x.to_ne_bytes().to_vec(),
                    (Datatype::F64, DatatypeValue::F64(x)) =>x.to_ne_bytes().to_vec(),
                    (Datatype::EID, DatatypeValue::EID(x)) =>x.to_ne_bytes().to_vec(),
                    (Datatype::B256, DatatypeValue::B256(x)) => x.as_bytes().to_vec(),
                    (Datatype::COMP(_), DatatypeValue::COMP) => vec![],
                    _ => {flag = true; vec![]}
                };
                if flag {return Err("[Error][brick_editor.rs][set_field] Field datatype doesn't match with given datatype.".to_string()); }
               
                if let Some(component_type) = self.engine.get_component_type(brick.component) {
                    let offset = field_editor.brick_editor.get_field_offset(&component_type, field.name).unwrap();
                    let offset_bytesize = offset + field.datatype.bytesize(self.engine);
                    println!("Brick data 1 = {:?}", brick.data);
                    brick.data.drain(offset..offset_bytesize); 
                    println!("Brick data 2 = {:?}", brick.data);
                    brick.data.splice(offset..offset, value);
                    self.engine.entity_brick_storage.lock().unwrap().insert(self.brick_id, brick);
                    return Ok(());
                }
            }
            return  Err(format!("[Error][brick_editor.rs][set_field] brick_id {} not found in engine.", self.brick_id));
        }
        return  Err(format!("[Error][brick_editor.rs][set_field] Cannot create brick editor for '{}' field name.", field.name));
   
    }

    fn get_field_offset(&self, component_type: &ComponentType, field_name: S32) -> Option<usize> {
        fn calculate(
            fields: &Vec<ComponentField>,
            field_name: &S32,
            engine: &EngineState,
        ) -> Option<usize> {
            let (offsets, contains) = fields.iter().fold(
                (vec![], false),
                |(mut old, flag), ComponentField { datatype, name }| {
                    let value = datatype.bytesize(engine);
                    match flag {
                        true => (old, flag),
                        false if name == field_name => (old, true),
                        false => {
                            old.push(value);
                            (old, flag)
                        }
                    }
                },
            );
            if !contains {
                None
            } else {
                Some(offsets.into_iter().sum())
            }
        }
        match component_type {
            ComponentType::Alias(field) if field.name == "self".into() => Some(0),
            ComponentType::Sum { fields, .. } => calculate(fields, &field_name, &self.engine),
            ComponentType::Product { fields, .. } => calculate(fields, &field_name, &self.engine),
            _ => None,
        }
    }
}

///
///
///
pub trait BrickEditing {
    ///
    fn get_brick_editor(&self, brick_id: EntityId) -> Option<BrickEditor>;
}

impl BrickEditing for EngineState {
    fn get_brick_editor(&self, brick_id: EntityId) -> Option<BrickEditor> {
        Some(BrickEditor {
            brick_id,
            engine: self,
        })
    }
}
/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod brick_editor_testing {

    use crate::{
        internals::{ComponentField, ComponentType, Datatype, EngineState},
        layers::{querying::Querying, brick_editor::DatatypeValue},
    };

    use super::BrickEditing;

    #[test]
    fn test_read_complex_type_field_data() {
        let engine_state = EngineState::default();

        let component_type = ComponentType::Product {
            name: "Position".into(),
            fields: vec![
                ComponentField {
                    name: "x".into(),
                    datatype:  Datatype::F32,
                },
                ComponentField {
                    name: "y".into(),
                    datatype: Datatype::F64,
                },
            ],
        };
        engine_state.add_component_type(component_type.clone());
        let a = engine_state.create_object();
        let input = {
            let mut buffer: Vec<u8> = vec![];
            buffer.extend(7.5f32.to_ne_bytes());
            buffer.extend(66.3f64.to_ne_bytes());
            buffer
        };
        engine_state.add_incoming_property(a, "Position".into(), input);
        let query = engine_state
            .query_entities()
            .with_target(a)
            .with_component("Position".into())
            .get();

        if let Some(&brick_id) = query.as_slice().first() {
            let brick_editor = engine_state.get_brick_editor(brick_id).unwrap();
            {
                let field_editor = brick_editor.get_field_editor("x".into()).unwrap();
                let offset = brick_editor.get_field_offset(&component_type, "x".into()).unwrap();
                assert_eq!(0, offset);
                let field_value = &field_editor.data;
                
                println!("x field value {:?}", field_value);
                assert_eq!(&DatatypeValue::F32(7.5), field_value);
            }
            {
                let field_editor = brick_editor.get_field_editor("y".into()).unwrap();
            
                let offset = brick_editor.get_field_offset(&component_type, "y".into()).unwrap();
                assert_eq!(4, offset);
                let field_value = &field_editor.data;
                println!("y field value {:?}", field_value);
                assert_eq!(&DatatypeValue::F64(66.3), field_value);
              
            }
            
        }
    }
    #[test]
    fn test_write_complex_type_field_data() {
        let engine_state = EngineState::default();

        let component_type = ComponentType::Product {
            name: "Position".into(),
            fields: vec![
                ComponentField {
                    name: "x".into(),
                    datatype: Datatype::F32,
                },
                ComponentField {
                    name: "y".into(),
                    datatype: Datatype::F64,
                },
            ],
        };
        engine_state.add_component_type(component_type.clone());
        let a = engine_state.create_object();
        let input = {
            let mut buffer: Vec<u8> = vec![];
            buffer.extend(7.5f32.to_ne_bytes());
            buffer.extend(66.3f64.to_ne_bytes());
            buffer
        };
        engine_state.add_incoming_property(a, "Position".into(), input);
        let query = engine_state
            .query_entities()
            .with_target(a)
            .with_component("Position".into())
            .get();

        if let Some(&brick_id) = query.as_slice().first() {
            let brick_editor = engine_state.get_brick_editor(brick_id).unwrap();
            {
                let comp_field = ComponentField {
                    name: "y".into(),
                    datatype: Datatype::F64,
                };
                let field_data = DatatypeValue::F64(777.5);
                println!("buffer field_data {:?}", field_data);
         
                let res = brick_editor.set_field(comp_field, field_data); 
                assert!(res.is_ok());
            }
            {
                let field_editor = brick_editor.get_field_editor("y".into()).unwrap();            
             
                let new_field_value = &field_editor.data;
                println!("'y' field value {:?}", new_field_value);
                assert_eq!(&DatatypeValue::F64(777.5), new_field_value);
              
            }
            
        }
    }
}
