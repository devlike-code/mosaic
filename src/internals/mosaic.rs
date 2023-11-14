use std::{
    collections::HashMap,
    ops::{Index, IndexMut, Range},
    sync::{Arc, Mutex},
    vec::IntoIter,
};

use atomic_counter::{AtomicCounter, RelaxedCounter};
use fstr::FStr;
use itertools::Itertools;
use ordered_multimap::ListOrderedMultimap;

use super::{
    logging::report_error, slice_into_array, ComponentType, DataBrick, Datatype, EntityId,
    EntityRegistry, SparseSet, Value, S32,
};

#[derive(PartialEq, Clone, Hash, Debug)]
pub enum TileType {
    Object,
    Arrow { source: EntityId, target: EntityId },
    Loop { endpoint: EntityId },
    Descriptor { subject: EntityId },
    Extension { subject: EntityId },
    Backlink { source: EntityId, target: EntityId },
}

#[derive(PartialEq, Clone, Debug)]
pub struct Tile {
    id: EntityId,
    tile_type: TileType,
    component: S32,
    data: HashMap<S32, Value>,
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
        entity_registry: Arc<EntityRegistry>,
        component_type: &ComponentType,
        field_name: S32,
    ) -> Option<Range<usize>> {
        entity_registry
            .component_offset_size_map
            .lock()
            .unwrap()
            .get(&(component_type.name(), field_name))
            .cloned()
    }

    fn create_fields_from_data_brick(
        entity_registry: Arc<EntityRegistry>,
        brick: &DataBrick,
    ) -> Result<HashMap<S32, Value>, String> {
        let component_type = entity_registry.get_component_type(brick.component)?;
        let component_fields = component_type
            .get_fields()
            .iter()
            .map(|field| {
                (
                    field.name,
                    field.datatype.to_owned(),
                    Self::get_field_offset(
                        Arc::clone(&entity_registry),
                        &component_type,
                        field.name,
                    ),
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
                    Datatype::B128 => Value::B128(FStr::<128>::from_str_lossy(
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
                return report_error(format!(
                    "Cannot create field {} from component data - field missing in component {}.",
                    field_name,
                    component_type.name()
                ));
            }
        }

        Ok(result)
    }

    fn create_binary_data_from_fields(&self, component: &ComponentType) -> Vec<u8> {
        let data = component
            .get_fields()
            .into_iter()
            .map(|f| self.data.get(&f.name).unwrap())
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
                    Value::B128(x) => x.as_bytes().to_vec(),
                };
                temp.extend(value_bytes);
                temp.resize(200, 0);
                temp
            });

        assert_eq!(200, data.len());
        data
    }

    pub fn commit(&self, entity_registry: Arc<EntityRegistry>) -> Result<(), String> {
        let component = entity_registry.get_component_type(self.component)?;
        let mut slab_storage = entity_registry.component_slabs.lock().unwrap();
        let slab = slab_storage.get_mut(&self.component).unwrap();

        let id_alloc = entity_registry.id_allocation_index.lock();

        if let Some(alloc) = id_alloc.unwrap().get(&self.id) {
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

            entity_registry
                .id_allocation_index
                .lock()
                .unwrap()
                .insert(self.id, alloc);
        }

        Ok(())
    }
}

impl Tile {
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

#[derive(Debug)]
pub struct Mosaic {
    entity_counter: Arc<RelaxedCounter>,
    entity_registry: Arc<EntityRegistry>,
    tile_registry: Mutex<HashMap<EntityId, Tile>>,
    dependent_ids_map: Mutex<ListOrderedMultimap<EntityId, EntityId>>,
    object_ids: Mutex<SparseSet>,
    arrow_ids: Mutex<SparseSet>,
    loop_ids: Mutex<SparseSet>,
    descriptor_ids: Mutex<SparseSet>,
    extension_ids: Mutex<SparseSet>,
}

impl PartialEq for Mosaic {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl Eq for Mosaic {}

impl Mosaic {
    fn new() -> Arc<Mosaic> {
        Arc::new(Mosaic {
            entity_counter: Arc::new(RelaxedCounter::default()),
            entity_registry: Arc::new(EntityRegistry::default()),
            tile_registry: Mutex::new(HashMap::default()),
            dependent_ids_map: Mutex::new(ListOrderedMultimap::default()),
            object_ids: Mutex::new(SparseSet::default()),
            arrow_ids: Mutex::new(SparseSet::default()),
            loop_ids: Mutex::new(SparseSet::default()),
            descriptor_ids: Mutex::new(SparseSet::default()),
            extension_ids: Mutex::new(SparseSet::default()),
        })
    }

    fn next_id(&self) -> EntityId {
        self.entity_counter.inc()
    }
}

trait MosaicCreateObject {
    fn new_object(&self, component: S32) -> Tile;
}

trait MosaicCRUD<Id> {
    fn new_arrow(&self, source: &Id, target: &Id, component: S32) -> Tile;
    fn new_loop(&self, endpoint: &Id, component: S32) -> Tile;
    fn new_descriptor(&self, subject: &Id, component: S32) -> Tile;
    fn new_extension(&self, subject: &Id, component: S32) -> Tile;
}

impl MosaicCRUD<EntityId> for Mosaic {
    fn new_arrow(&self, source: &EntityId, target: &EntityId, component: S32) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*source, id);
        self.dependent_ids_map.lock().unwrap().append(*target, id);

        let tile = Tile {
            id,
            tile_type: TileType::Arrow {
                source: *source,
                target: *target,
            },
            component,
            data: HashMap::default(),
        };
        self.arrow_ids.lock().unwrap().add(id);
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
        tile
    }

    fn new_loop(&self, endpoint: &EntityId, component: S32) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*endpoint, id);

        let tile = Tile {
            id,
            tile_type: TileType::Loop {
                endpoint: *endpoint,
            },
            component,
            data: HashMap::default(),
        };
        self.loop_ids.lock().unwrap().add(id);
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
        tile
    }

    fn new_descriptor(&self, subject: &EntityId, component: S32) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*subject, id);

        let tile = Tile {
            id,
            tile_type: TileType::Descriptor { subject: *subject },
            component,
            data: HashMap::default(),
        };
        self.descriptor_ids.lock().unwrap().add(id);
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
        tile
    }

    fn new_extension(&self, subject: &EntityId, component: S32) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*subject, id);

        let tile = Tile {
            id,
            tile_type: TileType::Extension { subject: *subject },
            component,
            data: HashMap::default(),
        };
        self.extension_ids.lock().unwrap().add(id);
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
        tile
    }
}

impl MosaicCRUD<Tile> for Mosaic {
    fn new_arrow(&self, source: &Tile, target: &Tile, component: S32) -> Tile {
        <Mosaic as MosaicCRUD<EntityId>>::new_arrow(self, &source.id, &target.id, component)
    }

    fn new_loop(&self, endpoint: &Tile, component: S32) -> Tile {
        <Mosaic as MosaicCRUD<EntityId>>::new_loop(self, &endpoint.id, component)
    }

    fn new_descriptor(&self, subject: &Tile, component: S32) -> Tile {
        <Mosaic as MosaicCRUD<EntityId>>::new_descriptor(self, &subject.id, component)
    }

    fn new_extension(&self, subject: &Tile, component: S32) -> Tile {
        <Mosaic as MosaicCRUD<EntityId>>::new_extension(self, &subject.id, component)
    }
}

impl MosaicCreateObject for Mosaic {
    fn new_object(&self, component: S32) -> Tile {
        let id = self.next_id();
        let tile = Tile {
            id,
            tile_type: TileType::Object,
            component,
            data: HashMap::default(),
        };
        self.object_ids.lock().unwrap().add(id);
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
        tile
    }
}

struct GetDependentTilesIterator {
    mosaic: Arc<Mosaic>,
    items: Vec<Tile>,
}

impl GetDependentTilesIterator {
    fn new<I>(iter: I, mosaic: Arc<Mosaic>) -> Self
    where
        I: Iterator<Item = Tile>,
    {
        let tile_storage = mosaic.tile_registry.lock().unwrap();
        let mut result = vec![];
        for item in iter {
            result.push(
                mosaic
                    .dependent_ids_map
                    .lock()
                    .unwrap()
                    .get_all(&item.id)
                    .filter_map(|id| tile_storage.get(id))
                    .rev()
                    .cloned()
                    .collect_vec(),
            )
        }

        GetDependentTilesIterator {
            mosaic: Arc::clone(&mosaic),
            items: result.into_iter().flatten().collect_vec(),
        }
    }
}

impl Iterator for GetDependentTilesIterator {
    type Item = Tile;

    fn next(&mut self) -> Option<Self::Item> {
        self.items.pop()
    }
}

trait GetDependentTilesExtension: Iterator {
    fn get_dependent_tiles(self, mosaic: Arc<Mosaic>) -> GetDependentTilesIterator;
}

impl<I> GetDependentTilesExtension for I
where
    I: Iterator<Item = Tile>,
{
    fn get_dependent_tiles(self, mosaic: Arc<Mosaic>) -> GetDependentTilesIterator {
        GetDependentTilesIterator::new(self, mosaic)
    }
}

impl Mosaic {
    pub fn get_dependent_tiles(&self, id: EntityId) -> IntoIter<Tile> {
        let tile_storage = self.tile_registry.lock().unwrap();
        self.dependent_ids_map
            .lock()
            .unwrap()
            .get_all(&id)
            .filter_map(|id| tile_storage.get(id))
            .cloned()
            .collect_vec()
            .into_iter()
    }

    pub fn get_objects(&self) -> IntoIter<Tile> {
        let tile_storage = self.tile_registry.lock().unwrap();
        self.object_ids
            .lock()
            .unwrap()
            .elements()
            .iter()
            .filter_map(|id| tile_storage.get(id))
            .cloned()
            .collect_vec()
            .into_iter()
    }

    pub fn get_arrows(&self) -> IntoIter<Tile> {
        let tile_storage = self.tile_registry.lock().unwrap();
        self.arrow_ids
            .lock()
            .unwrap()
            .elements()
            .iter()
            .filter_map(|id| tile_storage.get(id))
            .cloned()
            .collect_vec()
            .into_iter()
    }

    pub fn get_loops(&self) -> IntoIter<Tile> {
        let tile_storage = self.tile_registry.lock().unwrap();
        self.loop_ids
            .lock()
            .unwrap()
            .elements()
            .iter()
            .filter_map(|id| tile_storage.get(id))
            .cloned()
            .collect_vec()
            .into_iter()
    }

    pub fn get_descriptors(&self) -> IntoIter<Tile> {
        let tile_storage = self.tile_registry.lock().unwrap();
        self.descriptor_ids
            .lock()
            .unwrap()
            .elements()
            .iter()
            .filter_map(|id| tile_storage.get(id))
            .cloned()
            .collect_vec()
            .into_iter()
    }

    pub fn get_extensions(&self) -> IntoIter<Tile> {
        let tile_storage = self.tile_registry.lock().unwrap();
        self.extension_ids
            .lock()
            .unwrap()
            .elements()
            .iter()
            .filter_map(|id| tile_storage.get(id))
            .cloned()
            .collect_vec()
            .into_iter()
    }
}

#[cfg(test)]
mod mosaic_tests {
    use crate::internals::mosaic::{GetDependentTilesExtension, MosaicCreateObject};

    use super::{Mosaic, MosaicCRUD};

    #[test]
    fn test() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("Tile".into());
        let b = mosaic.new_object("Tile".into());
        let c = mosaic.new_arrow(&a, &b, "Tile".into());
        println!("{:?}", mosaic.dependent_ids_map.lock().unwrap());

        let d = mosaic.new_arrow(&b, &a, "Tile".into());

        println!("{:?}", mosaic.get_dependent_tiles(a.id));

        for dep in vec![a].into_iter().get_dependent_tiles(mosaic) {
            println!("{:?}", dep);
        }
    }
}
