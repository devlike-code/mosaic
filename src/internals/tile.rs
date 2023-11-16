use std::{
    collections::HashMap,
    ops::{Index, IndexMut, Range},
    sync::Arc,
};

use itertools::Itertools;

use crate::iterators::{
    get_arrows::{GetArrows, GetArrowsIterator},
    get_dependents::{GetDependentTiles, GetDependentsIterator},
    get_descriptors::{GetDescriptors, GetDescriptorsIterator},
    get_extensions::{GetExtensions, GetExtensionsIterator},
    just_tile::JustTileIterator,
};

use super::{
    logging::Logging, slice_into_array, ComponentType, DataBrick, Datatype, EntityId, Mosaic,
    MosaicCRUD, Value, S32,
};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Hash, Debug)]
pub enum TileType {
    Object,
    Arrow { source: EntityId, target: EntityId },
    Loop { endpoint: EntityId },
    Descriptor { subject: EntityId },
    Extension { subject: EntityId },
    Backlink { source: EntityId, target: EntityId },
}

#[derive(Clone, Debug)]
pub struct Tile {
    pub id: EntityId,
    pub tile_type: TileType,
    pub component: S32,
    pub data: HashMap<S32, Value>,
}

impl PartialEq for Tile {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Tile {}

impl PartialOrd for Tile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
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
        self.tile_type.hash(state);
        self.component.hash(state);
    }
}

impl Index<&str> for Tile {
    type Output = Value;
    fn index<'a>(&'a self, i: &str) -> &'a Value {
        self.data.get(&i.into()).unwrap()
    }
}

impl IndexMut<&str> for Tile {
    fn index_mut<'a>(&'a mut self, i: &str) -> &'a mut Value {
        self.data.get_mut(&i.into()).unwrap()
    }
}

impl Tile {
    fn get_field_offset(
        mosaic: &Mosaic,
        component_type: &ComponentType,
        field_name: S32,
    ) -> Option<Range<usize>> {
        mosaic
            .entity_registry
            .component_offset_size_map
            .lock()
            .unwrap()
            .get(&(component_type.name(), field_name))
            .cloned()
    }

    pub(crate) fn create_data_fields(&mut self, mosaic: &Mosaic) -> anyhow::Result<()> {
        let component_type = mosaic.entity_registry.get_component_type(self.component)?;
        let component_fields = component_type
            .get_fields()
            .iter()
            .map(|field| {
                (
                    field.name,
                    field.datatype.to_owned(),
                    Self::get_field_offset(mosaic, &component_type, field.name),
                )
            })
            .collect_vec();

        for (field_name, datatype, _) in component_fields {
            let value: Value = match datatype {
                Datatype::VOID => Value::VOID,
                // COMP fields will disappear when the component is added to the engine state,
                // so this situation should never arise. However, we'll leave a log here just in case.
                Datatype::COMP(_) => Value::VOID,
                Datatype::I32 => Value::I32(0),
                Datatype::U32 => Value::U32(0),
                Datatype::F32 => Value::F32(0.0),
                Datatype::S32 => Value::S32("".into()),
                Datatype::I64 => Value::I64(0),
                Datatype::U64 => Value::U64(0),
                Datatype::F64 => Value::F64(0.0),
                Datatype::EID => Value::EID(0),
                Datatype::B128 => Value::B128(vec![]),
            };

            self.data.insert(
                if component_type.is_alias() {
                    "self".into()
                } else {
                    field_name
                },
                value,
            );
        }

        Ok(())
    }

    pub(crate) fn read_fields_from_data_brick(
        mosaic: &Mosaic,
        brick: &DataBrick,
    ) -> anyhow::Result<HashMap<S32, Value>> {
        let component_type = mosaic.entity_registry.get_component_type(brick.component)?;
        let component_fields = component_type
            .get_fields()
            .iter()
            .map(|field| {
                (
                    field.name,
                    field.datatype.to_owned(),
                    Self::get_field_offset(mosaic, &component_type, field.name),
                )
            })
            .collect_vec();

        let mut result = HashMap::default();
        for (field_name, datatype, field_offset) in component_fields {
            if let Some(offset) = field_offset {
                let field_data_raw = &brick.data[offset.clone()];

                let value: Value = match datatype {
                    Datatype::VOID => Value::VOID,
                    // COMP fields will disappear when the component is added to the engine state,
                    // so this situation should never arise. However, we'll leave a log here just in case.
                    Datatype::COMP(_) => Value::VOID,
                    Datatype::I32 => {
                        Value::I32(i32::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::U32 => {
                        Value::U32(u32::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::F32 => {
                        Value::F32(f32::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::S32 => Value::S32(field_data_raw.into()),
                    Datatype::I64 => {
                        Value::I64(i64::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::U64 => {
                        Value::U64(u64::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::F64 => {
                        Value::F64(f64::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::EID => {
                        Value::EID(usize::from_be_bytes(slice_into_array(field_data_raw)))
                    }
                    Datatype::B128 => Value::B128(slice_into_array(field_data_raw)),
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
                return format!(
                    "Cannot create field {} from component data - field missing in component {}.",
                    field_name,
                    component_type.name()
                )
                .to_error();
            }
        }

        Ok(result)
    }

    fn create_binary_data_from_fields(&self, component: &ComponentType) -> Vec<u8> {
        let data = component
            .get_fields()
            .into_iter()
            .map(|f| {
                if component.is_alias() {
                    self.data.get(&"self".into()).unwrap()
                } else {
                    self.data.get(&f.name).unwrap()
                }
            })
            .fold(vec![], |old: Vec<u8>, value| {
                let mut temp = old.clone();
                let value_bytes: Vec<u8> = match value {
                    Value::VOID => vec![],
                    Value::I32(x) => x.to_be_bytes().to_vec(),
                    Value::U32(x) => x.to_be_bytes().to_vec(),
                    Value::F32(x) => x.to_be_bytes().to_vec(),
                    Value::S32(x) => x.0.as_bytes().to_vec(),
                    Value::I64(x) => x.to_be_bytes().to_vec(),
                    Value::U64(x) => x.to_be_bytes().to_vec(),
                    Value::F64(x) => x.to_be_bytes().to_vec(),
                    Value::EID(x) => x.to_be_bytes().to_vec(),
                    Value::B128(x) => x.clone(),
                };
                temp.extend(value_bytes);
                temp.resize(200, 0);
                temp
            });

        assert_eq!(200, data.len());
        data
    }

    pub fn commit(&self, mosaic: Arc<Mosaic>) -> anyhow::Result<()> {
        if !mosaic.tile_exists(&self.id) {
            return format!("Tile {} isn't valid.", self.id).to_error();
        }

        let component = mosaic.entity_registry.get_component_type(self.component)?;
        let mut slab_storage = mosaic.entity_registry.component_slabs.lock().unwrap();
        let slab = slab_storage.get_mut(&self.component).unwrap();

        let mut id_alloc = mosaic.entity_registry.id_allocation_index.lock().unwrap();

        if let Some(alloc) = id_alloc.get(&self.id) {
            let brick = slab.get_mut(*alloc).unwrap();

            brick
                .data
                .copy_from_slice(self.create_binary_data_from_fields(&component).as_slice());
        } else {
            let mut brick =
                DataBrick::new(self.id, self.source_id(), self.target_id(), self.component);
            brick
                .data
                .copy_from_slice(self.create_binary_data_from_fields(&component).as_slice());

            let alloc = slab.insert(brick);

            id_alloc.insert(self.id, alloc);
        }

        Ok(())
    }
}

impl Tile {
    pub fn new(mosaic: &Mosaic, id: EntityId, tile_type: TileType, component: S32) -> Tile {
        let mut tile = Tile {
            id,
            tile_type,
            component,
            data: HashMap::default(),
        };

        tile.create_data_fields(mosaic)
            .expect("Cannot create data fields, panicking!");

        tile
    }

    pub fn iter_with(&self, mosaic: &Arc<Mosaic>) -> JustTileIterator {
        JustTileIterator::new(Some(self.clone()), Arc::clone(mosaic))
    }

    pub fn source_id(&self) -> EntityId {
        match self.tile_type {
            TileType::Object => self.id,
            TileType::Arrow { source, .. } => source,
            TileType::Loop { endpoint } => endpoint,
            TileType::Descriptor { .. } => self.id,
            TileType::Extension { subject } => subject,
            TileType::Backlink { source, .. } => source,
        }
    }

    pub fn target_id(&self) -> EntityId {
        match self.tile_type {
            TileType::Object => self.id,
            TileType::Arrow { target, .. } => target,
            TileType::Loop { endpoint } => endpoint,
            TileType::Descriptor { subject } => subject,
            TileType::Extension { .. } => self.id,
            TileType::Backlink { target, .. } => target,
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.tile_type, TileType::Object)
    }

    pub fn is_arrow(&self) -> bool {
        matches!(self.tile_type, TileType::Arrow { .. })
    }

    pub fn is_loop(&self) -> bool {
        matches!(self.tile_type, TileType::Loop { .. })
    }

    pub fn is_descriptor(&self) -> bool {
        matches!(self.tile_type, TileType::Descriptor { .. })
    }

    pub fn is_extension(&self) -> bool {
        matches!(self.tile_type, TileType::Extension { .. })
    }

    pub fn is_backlink(&self) -> bool {
        matches!(self.tile_type, TileType::Backlink { .. })
    }
}

impl Tile {
    pub fn get_arrows_with(&self, mosaic: &Arc<Mosaic>) -> GetArrowsIterator {
        self.iter_with(mosaic).get_dependents().get_arrows()
    }

    pub fn get_dependents_with(&self, mosaic: &Arc<Mosaic>) -> GetDependentsIterator {
        self.iter_with(mosaic).get_dependents()
    }

    pub fn get_descriptors_with(&self, mosaic: &Arc<Mosaic>) -> GetDescriptorsIterator {
        self.iter_with(mosaic).get_dependents().get_descriptors()
    }

    pub fn get_extensions_with(&self, mosaic: &Arc<Mosaic>) -> GetExtensionsIterator {
        self.iter_with(mosaic).get_dependents().get_extensions()
    }
}
