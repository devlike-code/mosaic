use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    vec::IntoIter,
};

use atomic_counter::{AtomicCounter, RelaxedCounter};
use fstr::FStr;
use itertools::Itertools;
use ordered_multimap::ListOrderedMultimap;

use super::{
    slice_into_array, ComponentRegistry, ComponentValues, EntityId, Logging, SparseSet, Tile,
    TileType, ToByteArray, S32,
};

#[derive(Debug)]
pub struct Mosaic {
    pub(crate) entity_counter: Arc<RelaxedCounter>,
    pub(crate) component_registry: Arc<ComponentRegistry>,
    pub(crate) tile_registry: Mutex<HashMap<EntityId, Tile>>,
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
    pub fn new() -> Arc<Mosaic> {
        let mosaic = Arc::new(Mosaic {
            entity_counter: Arc::new(RelaxedCounter::default()),
            component_registry: Arc::new(ComponentRegistry::default()),
            tile_registry: Mutex::new(HashMap::default()),
            dependent_ids_map: Mutex::new(ListOrderedMultimap::default()),
            object_ids: Mutex::new(SparseSet::default()),
            arrow_ids: Mutex::new(SparseSet::default()),
            descriptor_ids: Mutex::new(SparseSet::default()),
            extension_ids: Mutex::new(SparseSet::default()),
        });

        mosaic.new_type("String: s128;").unwrap();
        mosaic.new_type("Group: s32;").unwrap();
        mosaic.new_type("GroupOwner: s32;").unwrap();

        mosaic.new_type("Process: s32;").unwrap();
        mosaic.new_type("ProcessParameter: s32;").unwrap();
        mosaic.new_type("ParameterBinding: s32;").unwrap();
        mosaic.new_type("ProcessResult: unit;").unwrap();
        mosaic.new_type("ResultBinding: unit;").unwrap();
        mosaic
            .new_type("Error: { position: s32, message: s128 };")
            .unwrap();

        mosaic.new_type("DEBUG: unit;").unwrap();

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

#[derive(Debug)]
pub(crate) enum MosaicLoadCommand {
    AddType(String),
    CreateTile(EntityId, EntityId, EntityId, S32, Vec<u8>),
}

pub trait MosaicIO {
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
    }

    Ok(result)
}

impl MosaicIO for Arc<Mosaic> {
    fn save(&self) -> Vec<u8> {
        let mut result = vec![];

        self.component_registry
            .component_definitions
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .sorted()
            .for_each(|v| {
                result.extend((v.len() as u16).to_be_bytes());
                result.extend(v.as_bytes());
            });

        result.extend(0u16.to_be_bytes());

        let mut entries = self
            .tile_registry
            .lock()
            .unwrap()
            .clone()
            .into_iter()
            .collect_vec();

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

    fn load(&self, data: &[u8]) -> anyhow::Result<()> {
        let offset = self.entity_counter.get();
        let loaded = load_mosaic_commands(data)?;

        loaded.into_iter().for_each(|command| match command {
            MosaicLoadCommand::AddType(definition) => {
                self.component_registry
                    .add_component_types(definition.as_str())
                    .unwrap();
            }
            MosaicLoadCommand::CreateTile(id, src, tgt, component, data) => {
                let id = id + offset;
                let src = src + offset;
                let tgt = tgt + offset;
                let component_type = &self
                    .component_registry
                    .get_component_type(component)
                    .unwrap();

                let fields = Tile::create_fields_from_binary_data(self, component_type, data);

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
            }
        });

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
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
        tile
    }

    fn new_specific_object(&self, id: EntityId, component: &str) -> anyhow::Result<Tile> {
        let mut registry = self.tile_registry.lock().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = registry.entry(id) {
            let tile = Tile {
                id,
                mosaic: Arc::clone(self),
                tile_type: TileType::Object,
                component: component.into(),
                data: HashMap::default(),
            };
            self.object_ids.lock().unwrap().add(id);
            e.insert(tile.clone());
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
        self.component_registry.add_component_types(type_def)
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
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
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
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
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
        self.tile_registry.lock().unwrap().insert(id, tile.clone());
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

        self.dependent_ids_map.lock().unwrap().remove(&id);
        if let Some(tile) = self.tile_registry.lock().unwrap().get(&id) {
            match tile.tile_type {
                TileType::Object => self.object_ids.lock().unwrap().remove(id),
                TileType::Arrow { .. } => self.arrow_ids.lock().unwrap().remove(id),
                TileType::Descriptor { .. } => self.descriptor_ids.lock().unwrap().remove(id),
                TileType::Extension { .. } => self.extension_ids.lock().unwrap().remove(id),
            }
        }

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
