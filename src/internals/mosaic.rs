use std::{
    collections::HashMap,
    ops::{Index, IndexMut, Range},
};

use array_tool::vec::Uniq;
use fstr::FStr;

use super::{
    datatypes::{EntityId, S32},
    slice_into_array, ComponentType, Datatype, DatatypeValue, EngineState,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// Bricks are the essential building blocks and hold a single component.
/// Every brick contains a single morphism and associated data
pub struct DataBrick {
    /// Identity of this element
    pub id: EntityId,
    /// The source element of this morphism
    pub source: EntityId,
    /// The target element of this morphism
    pub target: EntityId,
    /// The name of the component representing the data in this morphism
    pub component: S32,
    /// The actual data carried by the morphism
    pub data: Vec<u8>,
}

impl DataBrick {
    /// Updates the brick in the engine, lifting any changes into it
    fn update(&self, engine_state: &EngineState) {
        let mut storage = engine_state.entity_brick_storage.lock().unwrap();
        storage.insert(self.id, self.clone());
    }

    /// Refreshes the data from the engine into the brick; it doesn't touch anything other in the brick
    fn refresh(&mut self, engine_state: &EngineState) {
        let storage = engine_state.entity_brick_storage.lock().unwrap();
        self.data = storage.get(&self.id).unwrap_or(self).data.clone();
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TileData {
    component: S32,
    pub(crate) fields: HashMap<S32, DatatypeValue>,
}

#[derive(PartialEq, Clone)]
pub enum Tile {
    Object {
        id: EntityId,
        data: TileData,
    },
    Arrow {
        id: EntityId,
        source: EntityId,
        target: EntityId,
        data: TileData,
    },
    Loop {
        id: EntityId,
        origin: EntityId,
        data: TileData,
    },
    Descriptor {
        id: EntityId,
        target: EntityId,
        data: TileData,
    },
    Extension {
        id: EntityId,
        origin: EntityId,
        data: TileData,
    },
    Backlink {
        id: EntityId,
        source: EntityId,
        target: EntityId,
        data: TileData,
    },
}

impl Index<&str> for Tile {
    type Output = DatatypeValue;
    fn index<'a>(&'a self, i: &str) -> &'a DatatypeValue {
        println!("index data: {:?}", self.get_data());
        self.get_data().fields.get(&i.into()).unwrap()
    }
}

impl IndexMut<&str> for Tile {
    fn index_mut<'a>(&'a mut self, i: &str) -> &'a mut DatatypeValue {
        println!("index data mut: {:?}", self.get_data_mut());

        self.get_data_mut().fields.get_mut(&i.into()).unwrap()
    }
}

impl Tile {
    pub fn is_arrow(&self) -> bool {
        matches!(self, Tile::Arrow { .. })
    }
    
    pub fn is_object(&self) -> bool {
        matches!(self, Tile::Object { .. })
    }

    pub fn is_loop(&self) -> bool {
        matches!(self, Tile::Loop { .. })
    }

    pub fn is_descriptor(&self) -> bool {
        matches!(self, Tile::Descriptor { .. })
    }

    pub fn is_extension(&self) -> bool {
        matches!(self, Tile::Extension { .. })
    }

    pub fn is_property(&self) -> bool {
        self.is_descriptor() | self.is_extension()
    }

    pub fn set_field(&mut self, field: S32, field_data: DatatypeValue) {
        self.get_data_mut().fields.insert(field, field_data);
    }

    pub fn commit(&self, engine_state: &EngineState) -> Result<(), String> {
        let mut brick = engine_state.get_brick(self.id());
        let component = engine_state.get_component_type(brick.component)?;

        //order of saving needs to be correct and in component fields it is.
        brick.data = component
            .get_fields()
            .into_iter()
            .map(|f| self.get_data().fields.get(&f.name).unwrap())
            .fold(vec![], |old: Vec<u8>, value| {
                let mut temp = old.clone();
                let value_bytes: Vec<u8> = match value {
                    DatatypeValue::VOID => vec![],
                    DatatypeValue::I32(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::U32(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::F32(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::S32(x) => x.0.as_bytes().to_vec(),
                    DatatypeValue::I64(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::U64(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::F64(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::EID(x) => x.to_be_bytes().to_vec(),
                    DatatypeValue::B256(x) => x.as_bytes().to_vec(),
                };
                temp.extend(value_bytes);
                temp
            });

        //push cloned brick back to engine state
        Ok(brick.update(engine_state))
    }

    pub fn order(&self) -> usize {
        match self {
            Tile::Object { .. } => 0,
            Tile::Loop { .. } => 1,
            Tile::Arrow { .. } => 2,
            Tile::Descriptor { .. } => 3,
            Tile::Extension { .. } => 4,
            Tile::Backlink { .. } => 5,
        }
    }

    pub fn polarize_towards(self, e: EntityId) -> Self {
        match &self {
            Tile::Arrow {
                id,
                source,
                target,
                data,
            } if e == *target => Tile::Backlink {
                id: *id,
                source: *source,
                target: *target,
                data: data.clone(),
            },
            _ => self,
        }
    }
}

impl Tile {
    pub fn id(&self) -> EntityId {
        match self {
            Tile::Object { id, .. } => *id,
            Tile::Loop { id, .. } => *id,
            Tile::Arrow { id, .. } => *id,
            Tile::Descriptor { id, .. } => *id,
            Tile::Extension { id, .. } => *id,
            Tile::Backlink { id, .. } => *id,
        }
    }

    pub fn get_data(&self) -> &TileData {
        match self {
            Tile::Object { data, .. } => data,
            Tile::Loop { data, .. } => data,
            Tile::Arrow { data, .. } => data,
            Tile::Descriptor { data, .. } => data,
            Tile::Extension { data, .. } => data,
            Tile::Backlink { data, .. } => data,
        }
    }

    pub fn get_data_mut(&mut self) -> &mut TileData {
        match self {
            Tile::Object { data, .. } => data,
            Tile::Loop { data, .. } => data,
            Tile::Arrow { data, .. } => data,
            Tile::Descriptor { data, .. } => data,
            Tile::Extension { data, .. } => data,
            Tile::Backlink { data, .. } => data,
        }
    }

    pub fn origin(&self) -> EntityId {
        match self {
            Tile::Object { id, .. } => *id,
            Tile::Arrow { source, .. } => *source,
            Tile::Loop { origin, .. } => *origin,
            Tile::Descriptor { target: origin, .. } => *origin,
            Tile::Extension { origin, .. } => *origin,
            Tile::Backlink { source, .. } => *source,
        }
    }

    pub fn get_endpoints(&self) -> (EntityId, EntityId) {
        match self {
            Tile::Object { id, .. } => (*id, *id),
            Tile::Arrow { source, target, .. } => (*source, *target),
            Tile::Loop { origin, .. } => (*origin, *origin),
            Tile::Descriptor {
                target: origin, id, ..
            } => (*origin, *id),
            Tile::Extension { origin, id, .. } => (*id, *origin),
            Tile::Backlink { source, target, .. } => (*source, *target),
        }
    }
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Object { id, data } => f.write_fmt(format_args!(
                "[ Object     | {}: {} -> {} | {} {:?} ]",
                id, id, id, data.component, data.fields
            )),
            Self::Arrow {
                id,
                source,
                target,
                data,
            } => f.write_fmt(format_args!(
                "[ Arrow      | {}: {} -> {} | {} {:?} ]",
                id, source, target, data.component, data.fields
            )),
            Self::Backlink {
                id,
                source,
                target,
                data,
            } => f.write_fmt(format_args!(
                "[ Backlink   | {}: {} <- {} | {} {:?} ]",
                id, target, source, data.component, data.fields
            )),
            Self::Loop { id, origin, data } => f.write_fmt(format_args!(
                "[ Loop       | {}: {} -> {} | {} {:?} ]",
                id, origin, origin, data.component, data.fields
            )),
            Self::Descriptor {
                id,
                target: origin,
                data,
            } => f.write_fmt(format_args!(
                "[ Descriptor | {}: {} -> {} | {} {:?} ]",
                id, id, origin, data.component, data.fields
            )),
            Self::Extension { id, origin, data } => f.write_fmt(format_args!(
                "[ Extension  | {}: {} -> {} | {} {:?} ]",
                id, origin, id, data.component, data.fields
            )),
        }
    }
}

pub struct Block {
    pub tiles: Vec<Tile>,
}

impl Block {
    pub fn new() -> Block {
        Block { tiles: vec![] }
    }

    pub fn extend(&mut self, other: Block) {
        self.tiles.extend(other.tiles);
        self.tiles = self.tiles.unique();
    }
}

impl Into<Block> for Vec<Tile> {
    fn into(mut self) -> Block {
        self.sort_by(|a, b| (a.order(), a.id()).cmp(&(b.order(), b.id())));
        Block { tiles: self }
    }
}

impl std::fmt::Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tiles = self.tiles.clone();
        tiles.sort_by(|a, b| (a.order(), a.id()).cmp(&(b.order(), b.id())));
        tiles.iter().fold(Ok(()), |result, item| {
            result.and_then(|_| item.fmt(f).and_then(|_| f.write_str("\n")))
        })
    }
}

fn get_field_offset(
    engine: &EngineState,
    component_type: &ComponentType,
    field_name: S32,
) -> Option<Range<usize>> {
    let offset_size_index = engine.component_offset_size_index.lock().unwrap();
    offset_size_index
        .get(&(component_type.name(), field_name))
        .cloned()
}

fn create_fields_from_data(
    engine: &EngineState,
    brick: DataBrick,
) -> Result<HashMap<S32, DatatypeValue>, String> {
    let component_type = engine.get_component_type(brick.component)?;
    let component_fields = component_type
        .get_fields()
        .iter()
        .map(|field| {
            (
                field.name,
                field.datatype.to_owned(),
                get_field_offset(engine, &component_type, field.name),
            )
        })
        .collect::<Vec<_>>();

    let mut result = HashMap::default();
    for (field_name, datatype, field_offset) in component_fields {
        if let Some(offset) = field_offset {
            let field_data_raw = &brick.data[offset];

            let value: DatatypeValue = match datatype {
                Datatype::VOID => DatatypeValue::VOID,
                // COMP fields will disappear when the component is added to the engine state,
                // so this situation should never arise. However, we'll leave a log here just in case.
                Datatype::COMP(_) => DatatypeValue::VOID,
                Datatype::I32 => {
                    DatatypeValue::I32(i32::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::U32 => {
                    DatatypeValue::U32(u32::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::F32 => {
                    DatatypeValue::F32(f32::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::S32 => DatatypeValue::S32(field_data_raw.into()),
                Datatype::I64 => {
                    DatatypeValue::I64(i64::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::U64 => {
                    DatatypeValue::U64(u64::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::F64 => {
                    DatatypeValue::F64(f64::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::EID => {
                    DatatypeValue::EID(usize::from_be_bytes(slice_into_array(field_data_raw)))
                }
                Datatype::B256 => DatatypeValue::B256(FStr::<256>::from_str_lossy(
                    std::str::from_utf8(field_data_raw).unwrap(),
                    b'\0',
                )),
            };

            result.insert(
                if component_type.is_alias() {
                    "self".into()
                } else {
                    field_name
                },
                value,
            );
        } else {
            return Err(format!("[Error][mosaic.rs][create_fields_from_data] Cannot create field {} from component data - field missing in component {}.", field_name, component_type.name()));
        }
    }

    Ok(result)
}

impl From<(&EngineState, &DataBrick)> for Tile {
    fn from((engine, brick): (&EngineState, &DataBrick)) -> Self {
        let fields = create_fields_from_data(engine, brick.clone()).unwrap();
        match (brick.id, brick.source, brick.target) {
            // ID : ID -> ID
            (id, src, tgt) if id == src && src == tgt => Self::Object {
                id,
                data: TileData {
                    component: brick.component,
                    fields,
                },
            },

            // ID : ID -> TGT
            (id, src, tgt) if id == src && src != tgt => Self::Descriptor {
                id,
                target: tgt,
                data: TileData {
                    component: brick.component,
                    fields,
                },
            },

            // ID : SRC -> ID
            (id, src, tgt) if id == tgt && src != tgt => Self::Extension {
                id,
                origin: src,
                data: TileData {
                    component: brick.component,
                    fields,
                },
            },

            // ID : ID' -> ID'
            (id, src, tgt) if src == tgt && src != id => Self::Loop {
                id,
                origin: src,
                data: TileData {
                    component: brick.component,
                    fields,
                },
            },

            // ID : SRC -> TGT
            (id, src, tgt) => Self::Arrow {
                id,
                source: src,
                target: tgt,
                data: TileData {
                    component: brick.component,
                    fields,
                },
            },
        }
    }
}

pub trait BrickEditing {
    ///
    fn get_brick(&self, brick_id: EntityId) -> DataBrick;
}

impl BrickEditing for EngineState {
    fn get_brick(&self, brick_id: EntityId) -> DataBrick {
        return self
            .entity_brick_storage
            .lock()
            .unwrap()
            .get(&brick_id)
            .unwrap()
            .clone();
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod mosaic_testing {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    #[derive(Hash)]
    struct A {
        a: u8,
        b: u8,
        c: String,
    }

    #[test]
    fn hash_of_a() {
        let mut hasher = DefaultHasher::new();
        let a = A {
            a: b'c',
            b: b'a',
            c: format!("qweqweijwqeiofjwioefjwoeifjoiwefjewf"),
        };
        a.hash(&mut hasher);
        println!("{:?}", hasher.finish());
    }
}
