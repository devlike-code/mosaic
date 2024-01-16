use std::{collections::HashMap, sync::Arc, vec::IntoIter};

use anyhow::anyhow;
use itertools::Itertools;
use log::debug;

use crate::internals::{ComponentField, ToByteArray};

use super::{
    Bytesize, ComponentType, ComponentValues, Datatype, EntityId, Mosaic, MosaicCRUD, MosaicIO,
    Value, S32,
};
use crate::internals::byte_utilities::FromByteArray;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug, Copy)]
pub enum TileType {
    Object,
    Arrow { source: EntityId, target: EntityId },
    Descriptor { subject: EntityId },
    Extension { subject: EntityId },
}

#[derive(Clone)]
pub struct Tile {
    pub id: EntityId,
    pub mosaic: Arc<Mosaic>,
    pub tile_type: TileType,
    pub component: S32,
}

impl Tile {
    pub fn data(&self) -> Vec<(S32, Value)> {
        let storage = self.mosaic.data_storage.lock().unwrap();
        if let Some(e) = storage.get(&self.component.to_string()) {
            if let Some(h) = e.get(&self.id) {
                h.clone().iter().map(|(a, b)| (*a, b.clone())).collect_vec()
            } else {
                vec![]
            }
        } else {
            vec![]
        }
    }

    pub fn iter(&self) -> IntoIter<Tile> {
        vec![self.clone()].into_iter()
    }

    pub fn get(&self, index: &str) -> Value {
        if let Some(ct) = self
            .mosaic
            .component_registry
            .component_type_map
            .lock()
            .unwrap()
            .get(&self.component)
        {
            if let Some(field) = ct.get_field(index.into()) {
                if field.datatype == Datatype::UNIT {
                    return Value::UNIT;
                }
            }
        }

        let storage = self.mosaic.data_storage.lock().unwrap();
        if let Some(e) = storage.get(&self.component.to_string()) {
            if let Some(h) = e.get(&self.id) {
                if h.contains_key(&index.into()) {
                    h.get(&index.into()).unwrap().clone()
                } else {
                    panic!(
                        "Cannot find component {:?} in id {}",
                        self.component.to_string(),
                        self.id
                    );
                }
            } else {
                Value::UNIT
            }
        } else {
            Value::UNIT
        }
    }

    pub fn remove_component_data(&self) {
        let mut storage = self.mosaic.data_storage.lock().unwrap();
        if let Some(e) = storage.get_mut(&self.component.to_string()) {
            let _ = e.remove(&self.id);
        }
    }
}

impl IntoIterator for Tile {
    type Item = Tile;

    type IntoIter = std::vec::IntoIter<Tile>;

    fn into_iter(self) -> Self::IntoIter {
        vec![self].into_iter()
    }
}

impl std::fmt::Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mark = match self.tile_type {
            TileType::Object => "o".to_string(),
            TileType::Arrow { source, target } => format!("a {}->{}", source, target),
            TileType::Descriptor { subject } => format!("d->{}", subject),
            TileType::Extension { subject } => format!("e<-{}", subject),
        };
        f.write_fmt(format_args!("({}|{}: {})", self.id, mark, self.component))
    }
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn stringify(tile: &Tile, fields: Vec<ComponentField>) -> String {
            fields
                .iter()
                .map(|f| {
                    let mut f_name = f.name.to_string();
                    if f_name == tile.component.to_string() {
                        f_name = "self".to_string();
                    }
                    match f.datatype {
                        Datatype::UNIT => f.name.to_string(),
                        Datatype::I8 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_i8())
                        }
                        Datatype::I16 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_i16())
                        }
                        Datatype::I32 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_i32())
                        }
                        Datatype::I64 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_i64())
                        }
                        Datatype::U8 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_u8())
                        }
                        Datatype::U16 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_u16())
                        }
                        Datatype::U32 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_u32())
                        }
                        Datatype::U64 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_u64())
                        }
                        Datatype::F32 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_f32())
                        }
                        Datatype::F64 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_f64())
                        }
                        Datatype::S32 => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_s32())
                        }
                        Datatype::STR => {
                            format!("{}: {}", f.name, tile.get(f_name.as_str()).as_str())
                        }
                        Datatype::BOOL => {
                            format!(
                                "{}: {}",
                                f.name,
                                if tile.get(f_name.as_str()).as_bool() {
                                    "true".to_string()
                                } else {
                                    "false".to_string()
                                }
                            )
                        }
                        Datatype::COMP(_) => "".to_string(),
                    }
                })
                .join(", ")
        }

        let mark = match self.tile_type {
            TileType::Object => "o".to_string(),
            TileType::Arrow { source, target } => format!("a {}->{}", source, target),
            TileType::Descriptor { subject } => format!("d->{}", subject),
            TileType::Extension { subject } => format!("e<-{}", subject),
        };

        let data = if self.mosaic.is_tile_valid(&self.id) {
            let comp_type = self
                .mosaic
                .component_registry
                .get_component_type(self.component)
                .unwrap();
            debug!("{} => {:?}", comp_type.name(), comp_type.get_fields());
            stringify(self, comp_type.get_fields())
        } else {
            "err: !".to_string()
        };

        f.write_fmt(format_args!(
            "({}|{}:{}|{})",
            self.id, mark, self.component, data
        ))
    }
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Tile {}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

impl Ord for Tile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl std::hash::Hash for Tile {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Tile {
    pub(crate) fn set_field(&mut self, index: &str, value: Value) {
        let mut storage = self.mosaic.data_storage.lock().unwrap();
        if let Some(entities_by_component) = storage.get_mut(&self.component.to_string()) {
            if let Some(entity_by_field) = entities_by_component.get_mut(&self.id) {
                entity_by_field.insert(index.into(), value);
            } else {
                let mut hm = HashMap::new();
                hm.insert(index.into(), value);
                entities_by_component.insert(self.id, hm);
            }
        }
    }

    pub(crate) fn create_data_fields(&mut self, defaults: ComponentValues) -> anyhow::Result<()> {
        let mut defaults = defaults.into_iter().collect::<HashMap<_, _>>();

        let component_type = self
            .mosaic
            .component_registry
            .get_component_type(self.component)?;

        if defaults.is_empty() {
            if component_type.is_alias() {
                defaults.insert(
                    "self".into(),
                    component_type
                        .get_fields()
                        .first()
                        .unwrap()
                        .datatype
                        .get_default(),
                );
            } else {
                for field in component_type.get_fields() {
                    defaults.insert(field.name, field.datatype.get_default());
                }
            }
        }

        component_type
            .get_fields()
            .iter()
            .map(|field| (field.name, field.datatype.to_owned()))
            .for_each(|(field_name, datatype)| {
                let name = if component_type.is_alias() {
                    "self".into()
                } else {
                    field_name
                };

                if let Some(default_field) = defaults.get(&name) {
                    if datatype == default_field.get_datatype() {
                        let value = defaults
                            .get(&name)
                            .cloned()
                            .unwrap_or(datatype.get_default());

                        self.set_field(&name.to_string(), value);
                    }
                } else {
                    println!("MISSING FIELD {:?}", name);
                }
            });

        Ok(())
    }

    pub(crate) fn create_fields_from_binary_data(
        mosaic: &Mosaic,
        component: &ComponentType,
        data: Vec<u8>,
    ) -> anyhow::Result<HashMap<S32, Value>> {
        let result: anyhow::Result<(usize, HashMap<S32, Value>)> = component
            .get_fields()
            .into_iter()
            .map(|f| {
                if component.is_alias() {
                    ("self".into(), f.datatype)
                } else {
                    (f.name, f.datatype)
                }
            })
            .try_fold(
                (0usize, HashMap::<S32, Value>::new()),
                |(ptr, mut old), (name, datatype)| {
                    let size = datatype.bytesize(&mosaic.component_registry);
                    if data.len() >= ptr + size {
                        let comp_data = &data[ptr..ptr + size];

                        let value = match datatype {
                            Datatype::UNIT => Value::UNIT,
                            Datatype::I8 => Value::I8(i8::from_byte_array(comp_data)),
                            Datatype::I16 => Value::I16(i16::from_byte_array(comp_data)),
                            Datatype::I32 => Value::I32(i32::from_byte_array(comp_data)),
                            Datatype::I64 => Value::I64(i64::from_byte_array(comp_data)),
                            Datatype::U8 => Value::U8(u8::from_byte_array(comp_data)),
                            Datatype::U16 => Value::U16(u16::from_byte_array(comp_data)),
                            Datatype::U32 => Value::U32(u32::from_byte_array(comp_data)),
                            Datatype::U64 => Value::U64(u64::from_byte_array(comp_data)),
                            Datatype::F32 => Value::F32(f32::from_byte_array(comp_data)),
                            Datatype::F64 => Value::F64(f64::from_byte_array(comp_data)),
                            Datatype::S32 => Value::S32(S32::from_byte_array(comp_data)),
                            Datatype::STR => Value::STR(String::from_byte_array(comp_data)),
                            Datatype::BOOL => Value::BOOL(bool::from_byte_array(comp_data)),
                            Datatype::COMP(_) => panic!("Unreachable"),
                        };

                        old.insert(name, value);
                        Ok((ptr + size, old))
                    } else {
                        Err(anyhow!(
                            "Wrong data layout in component {:?} with field {} -- maybe it changed recently?",
                            component.name(), name,
                        ))
                    }
                },
            );

        result.map(|(_, fields)| fields)
    }

    pub(crate) fn create_binary_data_from_fields(&self, component: &ComponentType) -> Vec<u8> {
        component
            .get_fields()
            .into_iter()
            .map(|f| {
                if component.is_alias() {
                    ("self".into(), self.get("self"))
                } else {
                    (f.name, self.get(&f.name.to_string()))
                }
            })
            .fold(vec![], |old: Vec<u8>, (_, value)| {
                let mut temp = old.clone();

                // temp.extend(name.to_byte_array());

                let value_bytes: Vec<u8> = match value {
                    Value::UNIT => vec![],
                    Value::I8(x) => x.to_byte_array(),
                    Value::I16(x) => x.to_byte_array(),
                    Value::I32(x) => x.to_byte_array(),
                    Value::I64(x) => x.to_byte_array(),
                    Value::U8(x) => x.to_byte_array(),
                    Value::U16(x) => x.to_byte_array(),
                    Value::U32(x) => x.to_byte_array(),
                    Value::U64(x) => x.to_byte_array(),
                    Value::F32(x) => x.to_byte_array(),
                    Value::F64(x) => x.to_byte_array(),
                    Value::S32(x) => x.to_byte_array(),
                    Value::STR(x) => x.to_byte_array(),
                    Value::BOOL(x) => x.to_byte_array(),
                };
                temp.extend(value_bytes);
                temp
            })
    }
}

impl Tile {
    pub fn arrow_to(&self, other: &Tile, component: &str, data: ComponentValues) -> Tile {
        self.mosaic.new_arrow(self, other, component, data)
    }

    pub fn new(
        mosaic: Arc<Mosaic>,
        id: EntityId,
        tile_type: TileType,
        component: S32,
        fields: ComponentValues,
    ) -> Tile {
        let mut tile = Tile {
            id,
            mosaic: Arc::clone(&mosaic),
            tile_type,
            component,
        };

        tile.create_data_fields(fields)
            .expect("Cannot create data fields, panicking!");

        mosaic
            .tile_registry
            .lock()
            .unwrap()
            .insert(id, tile.clone());
        tile
    }

    pub fn source(&self) -> Tile {
        self.mosaic.get(self.source_id()).unwrap()
    }

    pub fn target(&self) -> Tile {
        self.mosaic.get(self.target_id()).unwrap()
    }

    pub fn source_id(&self) -> EntityId {
        match self.tile_type {
            TileType::Object => self.id,
            TileType::Arrow { source, .. } => source,
            TileType::Descriptor { .. } => self.id,
            TileType::Extension { subject } => subject,
        }
    }

    pub fn target_id(&self) -> EntityId {
        match self.tile_type {
            TileType::Object => self.id,
            TileType::Arrow { target, .. } => target,
            TileType::Descriptor { subject } => subject,
            TileType::Extension { .. } => self.id,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.tile_type, TileType::Object)
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self.tile_type, TileType::Arrow { .. })
    }

    pub fn is_loop(&self) -> bool {
        matches!(self.tile_type, TileType::Arrow { .. }) && self.source_id() == self.target_id()
    }

    pub fn is_descriptor(&self) -> bool {
        matches!(self.tile_type, TileType::Descriptor { .. })
    }

    pub fn is_extension(&self) -> bool {
        matches!(self.tile_type, TileType::Extension { .. })
    }
}
