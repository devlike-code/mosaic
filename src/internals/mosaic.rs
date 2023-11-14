use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    sync::{Arc, Mutex},
};

use atomic_counter::{AtomicCounter, RelaxedCounter};
use ordered_multimap::ListOrderedMultimap;

use super::{
    get_entities::GetEntitiesIterator, EntityId, EntityRegistry, Logging, SparseSet, Tile,
    TileType, S32,
};

#[derive(Debug)]
pub struct Mosaic {
    entity_counter: Arc<RelaxedCounter>,
    entity_registry: Arc<EntityRegistry>,
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
}

impl Mosaic {
    pub fn index(&self, i: EntityId) -> Option<Tile> {
        self.tile_registry.lock().unwrap().get(&i).cloned()
    }

    pub fn commit(&self, tile: &Tile) -> anyhow::Result<()> {
        tile.commit(Arc::clone(&self.entity_registry))
    }

    pub fn tile_exists(&self, i: EntityId) -> bool {
        self.tile_registry.lock().unwrap().contains_key(&i)
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
        if registry.contains_key(&id) {
            format!(
                "Cannot create specific object at id {}, it already exists:\n\t{:?}",
                id,
                self.index(id)
            )
            .to_error()
        } else {
            let tile = Tile {
                id,
                tile_type: TileType::Object,
                component,
                data: HashMap::default(),
            };
            self.object_ids.lock().unwrap().add(id);
            registry.insert(id, tile.clone());
            Ok(tile)
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

pub trait WithMosaic {
    fn get_mosaic(&self) -> Arc<Mosaic>;
}

#[cfg(test)]
mod mosaic_tests {
    use itertools::Itertools;

    use crate::{
        internals::get_entities::{GetEntities, GetEntitiesExtension},
        iterators::{get_dependent_tiles::GetDependentTiles, get_objects::GetObjects},
    };

    use super::{Mosaic, MosaicCRUD};

    #[test]
    fn test() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("Tile".into());
        let b = mosaic.new_object("Tile".into());
        let _c = mosaic.new_arrow(&a, &b, "Tile".into());
        let _d = mosaic.new_arrow(&b, &a, "Tile".into());

        for dep in a
            .iter_with(&mosaic)
            .get_entities()
            .get_objects()
            .get_dependents()
            .unique()
        {
            println!("{:?}", dep);
        }
    }
}
