use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use atomic_counter::{AtomicCounter, RelaxedCounter};
use itertools::Itertools;
use ordered_multimap::ListOrderedMultimap;

use super::{
    get_entities::GetEntitiesIterator, EntityId, EntityRegistry, Logging, SparseSet, Tile,
    TileType, S32,
};

#[derive(Debug)]
pub struct Mosaic {
    pub(crate) entity_counter: Arc<RelaxedCounter>,
    pub(crate) entity_registry: Arc<EntityRegistry>,
    pub(crate) tile_registry: Mutex<HashMap<EntityId, Tile>>,
    pub(crate) dependent_ids_map: Mutex<ListOrderedMultimap<EntityId, EntityId>>,
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
    pub fn new() -> Arc<Mosaic> {
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
        let registry = self.tile_registry.lock().unwrap();
        let mut id = self.entity_counter.inc();
        while registry.contains_key(&id) {
            id = self.entity_counter.inc();
        }
        id
    }
}

pub trait MosaicCRUD<Id> {
    // not generic in Id, but still a part:
    // fn new_object(&self, component: S32) -> Tile
    fn new_arrow(&self, source: &Id, target: &Id, component: S32) -> Tile;
    fn new_loop(&self, endpoint: &Id, component: S32) -> Tile;
    fn new_descriptor(&self, subject: &Id, component: S32) -> Tile;
    fn new_extension(&self, subject: &Id, component: S32) -> Tile;
    fn tile_exists(&self, i: &Id) -> bool;
    fn delete_tile(&self, tile: Id);
}

pub trait TileCommit {
    fn commit(&self, tile: &Tile) -> anyhow::Result<()>;
}

impl TileCommit for Arc<Mosaic> {
    fn commit(&self, tile: &Tile) -> anyhow::Result<()> {
        tile.commit(Arc::clone(self))
    }
}

impl Mosaic {
    pub fn get(&self, i: EntityId) -> Option<Tile> {
        self.tile_registry.lock().unwrap().get(&i).cloned()
    }

    pub fn new_object(&self, component: S32) -> Tile {
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

    pub fn new_specific_object(&self, id: EntityId, component: S32) -> anyhow::Result<Tile> {
        let mut registry = self.tile_registry.lock().unwrap();
        if let std::collections::hash_map::Entry::Vacant(e) = registry.entry(id) {
            let tile = Tile {
                id,
                tile_type: TileType::Object,
                component,
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
}

pub trait MosaicGetEntities {
    fn get_entities(&self) -> GetEntitiesIterator;
}

impl MosaicGetEntities for Arc<Mosaic> {
    fn get_entities(&self) -> GetEntitiesIterator {
        GetEntitiesIterator::new(Arc::clone(self))
    }
}

impl MosaicCRUD<EntityId> for Mosaic {
    fn tile_exists(&self, i: &EntityId) -> bool {
        self.tile_registry.lock().unwrap().contains_key(i)
    }

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
        let mut component = None;
        if let Some(tile) = self.tile_registry.lock().unwrap().get(&id) {
            component = Some(tile.component);
            match tile.tile_type {
                TileType::Object => self.object_ids.lock().unwrap().remove(id),
                TileType::Arrow { .. } | TileType::Backlink { .. } => {
                    self.arrow_ids.lock().unwrap().remove(id)
                }
                TileType::Loop { .. } => self.loop_ids.lock().unwrap().remove(id),
                TileType::Descriptor { .. } => self.descriptor_ids.lock().unwrap().remove(id),
                TileType::Extension { .. } => self.extension_ids.lock().unwrap().remove(id),
            }
        }

        self.tile_registry.lock().unwrap().remove(&id);

        if let Some(alloc) = self
            .entity_registry
            .id_allocation_index
            .lock()
            .unwrap()
            .get(&id)
        {
            self.entity_registry
                .component_slabs
                .lock()
                .unwrap()
                .get_mut(&component.unwrap())
                .unwrap()
                .remove(*alloc);
        }

        self.entity_registry
            .id_allocation_index
            .lock()
            .unwrap()
            .remove(&id);
    }
}

impl MosaicCRUD<Tile> for Mosaic {
    fn tile_exists(&self, i: &Tile) -> bool {
        <Mosaic as MosaicCRUD<EntityId>>::tile_exists(self, &i.id)
    }

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

    fn delete_tile(&self, tile: Tile) {
        <Mosaic as MosaicCRUD<EntityId>>::delete_tile(self, tile.id);
    }
}

pub trait WithMosaic {
    fn get_mosaic(&self) -> Arc<Mosaic>;
}
