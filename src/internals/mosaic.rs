use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
    vec::IntoIter,
};

use anyhow::anyhow;
use atomic_counter::{AtomicCounter, RelaxedCounter};
use fstr::FStr;
use itertools::Itertools;
use once_cell::sync::Lazy;
use ordered_multimap::ListOrderedMultimap;

use super::{
    slice_into_array, ComponentRegistry, ComponentValues, EntityId, Logging, SparseSet, Tile,
    TileType, ToByteArray, Value, S32,
};

type ComponentName = String;
type ComponentField = S32;
type DataStorage = HashMap<ComponentName, HashMap<EntityId, HashMap<ComponentField, Value>>>;

#[allow(clippy::type_complexity)]
pub static MOSAIC_INSTANCES: Lazy<Arc<Mutex<HashMap<usize, Arc<Mosaic>>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

#[derive(Debug)]
pub struct Mosaic {
    pub id: usize,
    pub(crate) entity_counter: RelaxedCounter,
    pub component_registry: ComponentRegistry,
    pub(crate) tile_registry: Mutex<HashMap<EntityId, Tile>>,
    pub(crate) data_storage: Mutex<DataStorage>,
    pub(crate) dependent_ids_map: Mutex<ListOrderedMultimap<EntityId, EntityId>>,
    object_ids: Mutex<SparseSet>,
    arrow_ids: Mutex<SparseSet>,
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
    pub fn dot(&self, name: &str) -> String {
        let tiles = {
            let reg = self.tile_registry.lock().unwrap();
            reg.values().cloned().collect_vec()
        };

        let horizontal = tiles.len() < 50;

        let mut output = vec![format!(
            "digraph {} {{\n\trankdir=\"{}\";\n",
            name,
            if horizontal { "TB" } else { "LR" }
        )];

        tiles.into_iter().for_each(|t| {
            let dt = format!("{:?}", t.component);
            if t.is_object() {
                output.push(format!("\t{} [label={:?}]", t.id, dt));
            } else if t.is_arrow() {
                output.push(format!(
                    "\t{} -> {} [label={:?}]",
                    t.source_id(),
                    t.target_id(),
                    dt
                ));
            } else if t.is_descriptor() {
                output.push(format!(
                    "\t{} -> {} [style=dashed, label={:?}]",
                    t.source_id(),
                    t.target_id(),
                    dt
                ));
            } else if t.is_extension() {
                output.push(format!(
                    "\t{} -> {} [style=dotted, label={:?}]",
                    t.source_id(),
                    t.target_id(),
                    dt
                ));
            }
        });

        output.push("}".to_string());
        output.join("\n")
    }

    pub fn new() -> Arc<Mosaic> {
        let id = { MOSAIC_INSTANCES.lock().unwrap().len() };

        let mosaic = Arc::new(Mosaic {
            id,
            entity_counter: RelaxedCounter::default(),
            component_registry: ComponentRegistry::default(),
            tile_registry: Mutex::new(HashMap::default()),
            dependent_ids_map: Mutex::new(ListOrderedMultimap::default()),
            data_storage: Mutex::new(HashMap::new()),
            object_ids: Mutex::new(SparseSet::default()),
            arrow_ids: Mutex::new(SparseSet::default()),
            descriptor_ids: Mutex::new(SparseSet::default()),
            extension_ids: Mutex::new(SparseSet::default()),
        });

        mosaic.new_type("void: unit;").unwrap();

        {
            MOSAIC_INSTANCES
                .lock()
                .unwrap()
                .insert(mosaic.id, Arc::clone(&mosaic));
        }
        mosaic
    }

    fn next_id(&self) -> EntityId {
        let registry = self.tile_registry.lock().unwrap();
        let mut id = self.entity_counter.inc();
        while registry.contains_key(&id) {
            id = self.entity_counter.inc();
        }
        id
    }
}

#[derive(Default)]
pub struct ComponentValuesBuilder {
    values: HashMap<S32, Value>,
}

pub fn par<T>(t: T) -> ComponentValues
where
    ComponentValuesBuilder: ComponentValuesBuilderSetter<T>,
{
    pars().is(t).ok()
}

pub fn pars() -> ComponentValuesBuilder {
    ComponentValuesBuilder::default()
}

impl ComponentValuesBuilder {
    pub fn ok(self) -> ComponentValues {
        self.values.into_iter().map(|(k, v)| (k, v)).collect_vec()
    }
}

pub trait ComponentValuesBuilderSetter<T>
where
    Self: std::marker::Sized,
{
    fn set(self, field: &str, value: T) -> ComponentValuesBuilder;

    fn is(self, value: T) -> ComponentValuesBuilder {
        self.set("self", value)
    }
}

impl ComponentValuesBuilderSetter<u8> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: u8) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::U8(value));
        self
    }
}

impl ComponentValuesBuilderSetter<u16> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: u16) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::U16(value));
        self
    }
}

impl ComponentValuesBuilderSetter<u32> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: u32) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::U32(value));
        self
    }
}

impl ComponentValuesBuilderSetter<u64> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: u64) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::U64(value));
        self
    }
}

impl ComponentValuesBuilderSetter<i8> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: i8) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::I8(value));
        self
    }
}

impl ComponentValuesBuilderSetter<i16> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: i16) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::I16(value));
        self
    }
}

impl ComponentValuesBuilderSetter<i32> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: i32) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::I32(value));
        self
    }
}

impl ComponentValuesBuilderSetter<i64> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: i64) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::I64(value));
        self
    }
}

impl ComponentValuesBuilderSetter<&str> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: &str) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::S32(value.into()));
        self
    }
}

impl ComponentValuesBuilderSetter<&[u8]> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: &[u8]) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::S128(value.into()));
        self
    }
}

impl ComponentValuesBuilderSetter<f32> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: f32) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::F32(value));
        self
    }
}

impl ComponentValuesBuilderSetter<f64> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: f64) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::F64(value));
        self
    }
}

impl ComponentValuesBuilderSetter<bool> for ComponentValuesBuilder {
    fn set(mut self, field: &str, value: bool) -> ComponentValuesBuilder {
        self.values.insert(field.into(), Value::BOOL(value));
        self
    }
}

pub trait MosaicTypelevelCRUD {
    fn new_type(&self, type_def: &str) -> anyhow::Result<()>;
}

pub trait MosaicCRUD<Id> {
    // not generic in Id, but still a part:
    // fn new_object(&self, component: &str) -> Tile
    fn new_arrow(
        &self,
        source: &Id,
        target: &Id,
        component: &str,
        defaults: ComponentValues,
    ) -> Tile;
    fn new_descriptor(&self, subject: &Id, component: &str, defaults: ComponentValues) -> Tile;
    fn new_extension(&self, subject: &Id, component: &str, defaults: ComponentValues) -> Tile;
    fn is_tile_valid(&self, i: &Id) -> bool;
    fn delete_tile(&self, tile: Id);
}

pub trait MosaicCopy<Id>: MosaicCRUD<Id> {
    fn copy_from(&self, from: &Self);
}

impl MosaicCopy<EntityId> for Arc<Mosaic> {
    fn copy_from(&self, from: &Self) {
        let mut mapping = HashMap::new();
        for foreign_entity in from.get_all() {
            let comp = foreign_entity.component.to_string();
            let data = foreign_entity.data();

            let local_entity = match foreign_entity.tile_type {
                TileType::Object => self.new_object(comp.as_str(), data),
                TileType::Arrow { source, target } => self.new_arrow(
                    mapping.get(&source).unwrap(),
                    mapping.get(&target).unwrap(),
                    comp.as_str(),
                    data,
                ),
                TileType::Descriptor { subject } => {
                    self.new_descriptor(mapping.get(&subject).unwrap(), comp.as_str(), data)
                }
                TileType::Extension { subject } => {
                    self.new_extension(mapping.get(&subject).unwrap(), comp.as_str(), data)
                }
            };

            mapping.insert(foreign_entity.id, local_entity.id);
        }
    }
}

pub trait TileGetById {
    fn get_tiles(&self, iter: Vec<EntityId>) -> IntoIter<Tile>;
}

impl TileGetById for Arc<Mosaic> {
    fn get_tiles(&self, iter: Vec<EntityId>) -> IntoIter<Tile> {
        iter.into_iter()
            .flat_map(|id| self.get(id))
            .collect_vec()
            .into_iter()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum MosaicLoadCommand {
    AddType(String),
    CreateTile(EntityId, EntityId, EntityId, S32, Vec<u8>),
}

pub trait MosaicIO {
    fn clear(&self);
    fn save(&self) -> Vec<u8>;
    fn load(&self, data: &[u8]) -> anyhow::Result<()>;
    fn get(&self, i: EntityId) -> Option<Tile>;
    fn get_all(&self) -> IntoIter<Tile>;
    fn new_object(&self, component: &str, defaults: ComponentValues) -> Tile;
    fn new_specific_object(&self, id: EntityId, component: &str) -> anyhow::Result<Tile>;
}

pub(crate) fn load_mosaic_commands(data: &[u8]) -> anyhow::Result<Vec<MosaicLoadCommand>> {
    let mut result = vec![];
    let mut ptr = 0usize;

    let total = data.len();

    loop {
        let len = u16::from_be_bytes(slice_into_array(&data[ptr..ptr + 2]));
        ptr += 2;
        if len == 0 {
            break;
        } else {
            let s = std::str::from_utf8(&data[ptr..ptr + len as usize]).unwrap();
            ptr += len as usize;
            result.push(MosaicLoadCommand::AddType(s.to_owned()));
        }
    }

    let mut types_used = HashSet::new();

    loop {
        if ptr == total {
            break;
        }

        let id = usize::from_be_bytes(slice_into_array(&data[ptr..ptr + 8]));
        ptr += 8;
        let src = usize::from_be_bytes(slice_into_array(&data[ptr..ptr + 8]));
        ptr += 8;
        let tgt = usize::from_be_bytes(slice_into_array(&data[ptr..ptr + 8]));
        ptr += 8;
        let comp_len = usize::from_be_bytes(slice_into_array(&data[ptr..ptr + 8]));
        ptr += 8;
        let comp_name = S32(FStr::<32>::from_str_lossy(
            std::str::from_utf8(&data[ptr..ptr + comp_len]).unwrap(),
            b'\0',
        ));
        ptr += comp_len;
        let comp_data_len = u32::from_be_bytes(slice_into_array(&data[ptr..ptr + 4]));
        ptr += 4;
        let comp_data = data[ptr..ptr + comp_data_len as usize].to_vec();
        ptr += comp_data_len as usize;

        result.push(MosaicLoadCommand::CreateTile(
            id, src, tgt, comp_name, comp_data,
        ));

        types_used.insert(comp_name.to_string());
    }

    result = result
        .iter()
        .flat_map(|command| match command {
            MosaicLoadCommand::AddType(t) if !types_used.contains(t.split(':').next().unwrap()) => {
                None
            }
            c => Some(c.clone()),
        })
        .collect_vec();
    Ok(result)
}

impl MosaicIO for Arc<Mosaic> {
    fn save(&self) -> Vec<u8> {
        let mut result = vec![];

        let mut entries = self
            .tile_registry
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .collect_vec();

        let used_types = entries
            .iter()
            .map(|(_, b)| b.component.to_string())
            .collect::<HashSet<_>>();

        println!("USED TYPES: {:?}", used_types);

        self.component_registry
            .component_definitions
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .filter(|c| used_types.contains(c.split(':').next().unwrap()))
            .sorted()
            .unique()
            .for_each(|v| {
                println!("Saving {:?}", v);
                result.extend((v.len() as u16).to_be_bytes());
                result.extend(v.as_bytes());
            });

        result.extend(0u16.to_be_bytes());

        entries.sort_by(|a, b| a.0.cmp(&b.0));

        entries.into_iter().for_each(|(_, t)| {
            result.extend(t.id.to_byte_array());
            result.extend(t.source_id().to_byte_array());
            result.extend(t.target_id().to_byte_array());
            let comp = t.component.0.as_str().replace('\0', "");
            result.extend(comp.len().to_byte_array());
            result.extend(comp.as_bytes());
            let data = t.create_binary_data_from_fields(
                &self
                    .component_registry
                    .get_component_type(t.component)
                    .unwrap(),
            );
            result.extend((data.len() as u32).to_byte_array());
            result.extend(data)
        });

        result
    }

    fn clear(&self) {
        self.tile_registry.lock().unwrap().clear();
        self.dependent_ids_map.lock().unwrap().clear();
        self.data_storage.lock().unwrap().clear();
        self.object_ids.lock().unwrap().clear();
        self.arrow_ids.lock().unwrap().clear();
        self.descriptor_ids.lock().unwrap().clear();
        self.extension_ids.lock().unwrap().clear();
        self.entity_counter.reset();
        self.component_registry.clear();
        self.new_type("void: unit;").unwrap();
    }

    fn load(&self, data: &[u8]) -> anyhow::Result<()> {
        let offset = self.entity_counter.get();
        let loaded = load_mosaic_commands(data)?;

        for command in loaded.into_iter() {
            match command {
                MosaicLoadCommand::AddType(definition) => {
                    let typename: S32 = definition
                        .split(':')
                        .collect_vec()
                        .first()
                        .unwrap()
                        .trim()
                        .into();

                    if !self.component_registry.has_component_type(&typename) {
                        self.component_registry
                            .add_component_types(definition.as_str())
                            .unwrap();
                    }
                }
                MosaicLoadCommand::CreateTile(id, src, tgt, component, data) => {
                    let id = id + offset;
                    let src = src + offset;
                    let tgt = tgt + offset;
                    let component_type = &self
                        .component_registry
                        .get_component_type(component)
                        .unwrap();

                    let field_access =
                        Tile::create_fields_from_binary_data(self, component_type, data);

                    if let Ok(fields) = field_access {
                        if id == src && id == tgt {
                            // ID : ID -> ID
                            let tile = Tile::new(
                                Arc::clone(self),
                                id,
                                TileType::Object,
                                component,
                                fields.into_iter().collect(),
                            );
                            self.object_ids.lock().unwrap().add(id);
                            self.tile_registry.lock().unwrap().insert(id, tile.clone());
                        } else if id == src && src != tgt {
                            // ID : ID -> TGT (descriptor)
                            self.dependent_ids_map.lock().unwrap().append(tgt, id);

                            let tile = Tile::new(
                                Arc::clone(self),
                                id,
                                TileType::Descriptor { subject: tgt },
                                component,
                                fields.into_iter().collect(),
                            );
                            self.descriptor_ids.lock().unwrap().add(id);
                            self.tile_registry.lock().unwrap().insert(id, tile.clone());
                        } else if id == tgt && src != tgt {
                            // ID : SRC -> ID (extension)
                            self.dependent_ids_map.lock().unwrap().append(src, id);

                            let tile = Tile::new(
                                Arc::clone(self),
                                id,
                                TileType::Extension { subject: src },
                                component,
                                fields.into_iter().collect(),
                            );
                            self.extension_ids.lock().unwrap().add(id);
                            self.tile_registry.lock().unwrap().insert(id, tile.clone());
                        } else {
                            self.dependent_ids_map.lock().unwrap().append(src, id);
                            self.dependent_ids_map.lock().unwrap().append(tgt, id);

                            let tile = Tile::new(
                                Arc::clone(self),
                                id,
                                TileType::Arrow {
                                    source: src,
                                    target: tgt,
                                },
                                component,
                                fields.into_iter().collect(),
                            );
                            self.arrow_ids.lock().unwrap().add(id);
                            self.tile_registry.lock().unwrap().insert(id, tile.clone());
                        }
                    } else {
                        return Err(field_access.unwrap_err());
                    }
                }
            }
        }

        Ok(())
    }

    fn get(&self, i: EntityId) -> Option<Tile> {
        self.tile_registry.lock().unwrap().get(&i).cloned()
    }

    fn new_object(&self, component: &str, defaults: ComponentValues) -> Tile {
        let id = self.next_id();
        let tile = Tile::new(
            Arc::clone(self),
            id,
            TileType::Object,
            component.into(),
            defaults,
        );
        self.object_ids.lock().unwrap().add(id);
        tile
    }

    fn new_specific_object(&self, id: EntityId, component: &str) -> anyhow::Result<Tile> {
        let mut registry = self.tile_registry.lock().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = registry.entry(id) {
            let mut tile = Tile {
                id,
                mosaic: Arc::clone(self),
                tile_type: TileType::Object,
                component: component.into(),
            };
            self.object_ids.lock().unwrap().add(id);
            e.insert(tile.clone());

            tile.create_data_fields(par(id.to_string().as_str()))
                .expect("Cannot create data fields, panicking!");

            Ok(tile)
        } else {
            format!(
                "Cannot create specific object at id {}, it already exists:\n\t{:?}",
                id,
                self.get(id)
            )
            .to_error()
        }
    }

    fn get_all(&self) -> IntoIter<Tile> {
        self.tile_registry
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect_vec()
            .into_iter()
    }
}

impl MosaicTypelevelCRUD for Arc<Mosaic> {
    fn new_type(&self, type_def: &str) -> anyhow::Result<()> {
        let d = type_def.to_string();
        let defs = d.chars().filter(|c| *c == ';').count();
        if defs > 1 {
            return Err(anyhow!(
                "Cannot have more than one type definition at once."
            ));
        }

        let type_name = d.split(':').collect_vec().first().cloned().unwrap();
        if self
            .component_registry
            .has_component_type(&type_name.into())
        {
            return Ok(());
        }

        let types = self.component_registry.add_component_types(type_def)?;
        let mut storage = self.data_storage.lock().unwrap();
        for typ in types {
            storage.insert(typ.name(), HashMap::new());
        }

        Ok(())
    }
}

impl MosaicCRUD<EntityId> for Arc<Mosaic> {
    fn is_tile_valid(&self, i: &EntityId) -> bool {
        self.tile_registry.lock().unwrap().contains_key(i)
    }

    fn new_arrow(
        &self,
        source: &EntityId,
        target: &EntityId,
        component: &str,
        defaults: ComponentValues,
    ) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*source, id);
        self.dependent_ids_map.lock().unwrap().append(*target, id);

        let tile = Tile::new(
            Arc::clone(self),
            id,
            TileType::Arrow {
                source: *source,
                target: *target,
            },
            component.into(),
            defaults,
        );
        self.arrow_ids.lock().unwrap().add(id);
        tile
    }

    fn new_descriptor(
        &self,
        subject: &EntityId,
        component: &str,
        defaults: ComponentValues,
    ) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*subject, id);

        let tile = Tile::new(
            Arc::clone(self),
            id,
            TileType::Descriptor { subject: *subject },
            component.into(),
            defaults,
        );
        self.descriptor_ids.lock().unwrap().add(id);
        tile
    }

    fn new_extension(
        &self,
        subject: &EntityId,
        component: &str,
        defaults: ComponentValues,
    ) -> Tile {
        let id = self.next_id();
        self.dependent_ids_map.lock().unwrap().append(*subject, id);

        let tile = Tile::new(
            Arc::clone(self),
            id,
            TileType::Extension { subject: *subject },
            component.into(),
            defaults,
        );
        self.extension_ids.lock().unwrap().add(id);
        tile
    }

    fn delete_tile(&self, id: EntityId) {
        let dependents = self
            .dependent_ids_map
            .lock()
            .unwrap()
            .get_all(&id)
            .cloned()
            .collect_vec();

        dependents.into_iter().for_each(|t| {
            self.delete_tile(t);
        });

        if !self.is_tile_valid(&id) {
            return;
        }

        let tile = self.get(id).unwrap();
        tile.remove_component_data();

        self.dependent_ids_map.lock().unwrap().remove(&id);
        if let Some(tile) = self.tile_registry.lock().unwrap().get(&id) {
            match tile.tile_type {
                TileType::Object => self.object_ids.lock().unwrap().remove(id),
                TileType::Arrow { .. } => self.arrow_ids.lock().unwrap().remove(id),
                TileType::Descriptor { .. } => self.descriptor_ids.lock().unwrap().remove(id),
                TileType::Extension { .. } => self.extension_ids.lock().unwrap().remove(id),
            }
        }
        //TODO! REMOVE FROM data_registry ALL component of entity
        //free id in freelist
        self.tile_registry.lock().unwrap().remove(&id);
    }
}

impl MosaicCRUD<Tile> for Arc<Mosaic> {
    fn is_tile_valid(&self, i: &Tile) -> bool {
        <Arc<Mosaic> as MosaicCRUD<EntityId>>::is_tile_valid(self, &i.id)
    }

    fn new_arrow(
        &self,
        source: &Tile,
        target: &Tile,
        component: &str,
        defaults: ComponentValues,
    ) -> Tile {
        <Arc<Mosaic> as MosaicCRUD<EntityId>>::new_arrow(
            self, &source.id, &target.id, component, defaults,
        )
    }

    fn new_descriptor(&self, subject: &Tile, component: &str, defaults: ComponentValues) -> Tile {
        <Arc<Mosaic> as MosaicCRUD<EntityId>>::new_descriptor(
            self,
            &subject.id,
            component,
            defaults,
        )
    }

    fn new_extension(&self, subject: &Tile, component: &str, defaults: ComponentValues) -> Tile {
        <Arc<Mosaic> as MosaicCRUD<EntityId>>::new_extension(self, &subject.id, component, defaults)
    }

    fn delete_tile(&self, tile: Tile) {
        <Arc<Mosaic> as MosaicCRUD<EntityId>>::delete_tile(self, tile.id);
    }
}
