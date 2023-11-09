use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use crate::layers::tiling::Tiling;

use super::{
    lifecycle::Lifecycle, ComponentType, EngineState, EntityId, SparseSet, Tile, Value,
    S32 as ComponentName,
};

pub struct MosaicEngine {
    pub(crate) engine_state: Arc<EngineState>,

    pub(crate) component_block_per_main_tile_index: HashMap<(EntityId, ComponentType), SparseSet>,
    pub(crate) archetype_per_tile_index: HashMap<EntityId, HashSet<ComponentType>>,
}

impl PartialEq for MosaicEngine {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

pub trait MosaicEngineExtension {
    fn register(&self, engine_state: Arc<EngineState>);
}

impl Default for MosaicEngine {
    fn default() -> Self {
        let engine_state = EngineState::new();
        engine_state
            .add_component_types("Object: void; Arrow: void; Label: s32; String: b256;")
            .unwrap();
        engine_state
            .add_component_types("Position: product { x: u32, y: u32 };")
            .unwrap();
        engine_state.add_component_types("Parent: void;").unwrap();
        Self {
            engine_state,
            component_block_per_main_tile_index: Default::default(),
            archetype_per_tile_index: Default::default(),
        }
    }
}

impl MosaicEngine {
    pub fn new() -> Arc<MosaicEngine> {
        Arc::new(MosaicEngine::default())
    }

    pub fn register_extension<E: MosaicEngineExtension>(&self, extension: E) {
        extension.register(Arc::clone(&self.engine_state));
    }
}

impl Lifecycle for Arc<MosaicEngine> {
    type Entity = Tile;

    fn create_object(&self, component: ComponentName, fields: Vec<Value>) -> Result<Tile, String> {
        let id = self.engine_state.create_object(component, fields)?;
        let tile = self.get_tile(id).ok_or(format!(
            "[Error][mosaic_engine.rs][create_object] Couldn't find tile with id {}",
            id
        ))?;

        Ok(tile)
    }

    fn create_arrow(
        &self,
        source: &Tile,
        target: &Tile,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Tile, String> {
        let id = self
            .engine_state
            .create_arrow(&source.id(), &target.id(), component, fields)?;
        let tile = self.get_tile(id).ok_or(format!(
            "[Error][mosaic_engine.rs][create_arrow] Couldn't find tile with id {}",
            id
        ))?;

        Ok(tile)
    }

    fn add_descriptor(
        &self,
        target: &Tile,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Tile, String> {
        let id = self
            .engine_state
            .add_descriptor(&target.id(), component, fields)?;
        let tile = self.get_tile(id).ok_or(format!(
            "[Error][mosaic_engine.rs][add_descriptor] Couldn't find tile with id {}",
            id
        ))?;

        Ok(tile)
    }

    fn add_extension(
        &self,
        target: &Tile,
        component: ComponentName,
        fields: Vec<Value>,
    ) -> Result<Tile, String> {
        let id = self
            .engine_state
            .add_extension(&target.id(), component, fields)?;
        let tile = self.get_tile(id).ok_or(format!(
            "[Error][mosaic_engine.rs][add_descriptor] Couldn't find tile with id {}",
            id
        ))?;

        Ok(tile)
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod mosaic_engine_testing {
    use std::clone;

    use itertools::Itertools;

    use crate::internals::{mosaic_engine::Lifecycle, Value, S32};

    use super::MosaicEngine;

    #[test]
    fn test_create_object() {
        let mosaic = MosaicEngine::new();
        let obj = mosaic.create_object("Object".into(), vec![]).unwrap();

        assert!(obj.is_object());
        assert_eq!(1, obj.id());
    }

    #[test]
    fn test_create_arrow() {
        let mosaic = MosaicEngine::new();
        let a = mosaic.create_object("Object".into(), vec![]).unwrap();
        let b = mosaic.create_object("Object".into(), vec![]).unwrap();
        let ab = mosaic.create_arrow(&a, &b, "Arrow".into(), vec![]).unwrap();

        assert!(ab.is_arrow());
        assert_eq!(3, ab.id());
        assert_eq!((a.id(), b.id()), ab.get_endpoints());
    }

    #[test]
    fn test_add_descriptor() {
        let mosaic = MosaicEngine::new();
        let obj = mosaic.create_object("Object".into(), vec![]).unwrap();
        let result = mosaic
            .add_descriptor(&obj, "Position".into(), vec![Value::U32(7), Value::U32(12)])
            .unwrap();
        assert_eq!(2, result.get_data().fields.len());
        assert_eq!(Value::U32(7), result["x"]);
        assert_eq!(Value::U32(12), result["y"]);
    }

    #[test]
    fn test_get_property() {
        let mosaic = MosaicEngine::new();
        let obj = mosaic.create_object("Object".into(), vec![]).unwrap();
        mosaic
            .add_descriptor(&obj, "Position".into(), vec![Value::U32(7), Value::U32(12)])
            .unwrap();

        let position = obj.get_property("Position".into());
        assert!(position.is_some());

        if let Some(pos) = position {
            assert_eq!(2, pos.get_data().fields.len());
            assert_eq!(Value::U32(7), pos["x"]);
            assert_eq!(Value::U32(12), pos["y"]);
        }
    }

    #[test]
    fn test_get_properties() {
        let mosaic = MosaicEngine::new();
        mosaic
            .engine_state
            .add_component_types("Log: s32;")
            .unwrap();
        let log_file = mosaic.create_object("Object".into(), vec![]).unwrap();

        log_file.add_descriptor("Log".into(), vec![Value::S32("Hello world".into())]);
        log_file.add_descriptor("Log".into(), vec![Value::S32("Log starts here".into())]);
        log_file.add_descriptor("Log".into(), vec![Value::S32("Entry 1".into())]);
        log_file.add_descriptor("Log".into(), vec![Value::S32("=========".into())]);

        log_file.add_extension(
            "Log".into(),
            vec![Value::S32("Extension added: 30min of play".into())],
        );

        let log = log_file.get_properties("Log".into());
        assert!(!log.is_empty());
        assert_eq!(5, log.len());
    }
}
