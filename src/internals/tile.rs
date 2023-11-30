use std::{collections::HashMap, sync::Arc, vec::IntoIter};

use crate::internals::ToByteArray;

use super::{
    Bytesize, ComponentType, ComponentValues, Datatype, EntityId, Mosaic, MosaicCRUD, MosaicIO,
    Value, S32,
};
use crate::internals::byte_utilities::FromByteArray;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
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
    pub data: HashMap<S32, Value>,
}

impl Tile {
    pub fn iter(&self) -> IntoIter<Tile> {
        vec![self.clone()].into_iter()
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
            TileType::Object => "x",
            TileType::Arrow { .. } => ">",
            TileType::Descriptor { .. } => "d",
            TileType::Extension { .. } => "e",
        };
        f.write_fmt(format_args!("({}|{})", mark, self.id))
    }
}

impl std::fmt::Debug for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mark = match self.tile_type {
            TileType::Object => "x".to_string(),
            TileType::Arrow { source, target } => format!("{} > {}", source, target),
            TileType::Descriptor { .. } => "d".to_string(),
            TileType::Extension { .. } => "e".to_string(),
        };
        f.write_fmt(format_args!(
            "({}|{}|{}|{:?})",
            mark, self.id, self.component, self.data
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
        self.data.insert(index.into(), value);
        self.mosaic
            .tile_registry
            .lock()
            .unwrap()
            .insert(self.id, self.clone());
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

                //when default name exists in component fields and field and default datatype is the same take the 'default' value
                //otherwise use field default value
                // println!("DEFAULTS {:?}", defaults);
                if let Some(default_field) = defaults.get(&name) {
                    if datatype == default_field.get_datatype() {
                        let value = defaults
                            .get(&name)
                            .cloned()
                            .unwrap_or(datatype.get_default());

                        // println!(
                        //     "field datatype:{:?}, default datatype: {:?}, default value: {:?}",
                        //     datatype,
                        //     default_field.get_datatype(),
                        //     value
                        // );

                        self.data.insert(name, value);
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
    ) -> HashMap<S32, Value> {
        let (_, fields) = component
            .get_fields()
            .into_iter()
            .map(|f| {
                if component.is_alias() {
                    ("self".into(), f.datatype)
                } else {
                    (f.name, f.datatype)
                }
            })
            .fold(
                (0usize, HashMap::<S32, Value>::new()),
                |(ptr, mut old), (name, datatype)| {
                    let size = datatype.bytesize(&mosaic.component_registry);
                    let comp_data = &data[ptr..ptr + size];

                    let value = match datatype {
                        Datatype::UNIT => Value::UNIT(()),
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
                        Datatype::S128 => Value::S128(comp_data.to_vec().clone()),
                        Datatype::BOOL => Value::BOOL(bool::from_byte_array(comp_data)),
                        Datatype::COMP(_) => panic!("Unreachable"),
                    };

                    old.insert(name, value);
                    (ptr + size, old)
                },
            );

        fields
    }
    pub(crate) fn create_binary_data_from_fields(&self, component: &ComponentType) -> Vec<u8> {
        component
            .get_fields()
            .into_iter()
            .map(|f| {
                if component.is_alias() {
                    ("self".into(), self.data.get(&"self".into()).unwrap())
                } else {
                    (f.name, self.data.get(&f.name).unwrap())
                }
            })
            .fold(vec![], |old: Vec<u8>, (_, value)| {
                let mut temp = old.clone();

                // temp.extend(name.to_byte_array());

                let value_bytes: Vec<u8> = match value {
                    Value::UNIT(()) => vec![],
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
                    Value::S128(x) => x.clone(),
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
            data: HashMap::default(),
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
