use std::sync::Arc;

use array_tool::vec::{Union, Uniq};
use itertools::Itertools;

use crate::internals::{
    mosaic_engine::MosaicEngine, query_iterator::QueryIterator, tile_iterator::TileIterator,
    EngineState, EntityId, Tile,
};

use super::tiling::Tiling;

pub trait Querying {
    type Entity;
    type CustomIterator;
    fn get_edges(&self, id: &Self::Entity) -> Self::CustomIterator;
    fn get_descriptors(&self, id: &Self::Entity) -> Self::CustomIterator;
    fn get_extensions(&self, id: &Self::Entity) -> Self::CustomIterator;
    fn get_properties(&self, id: &Self::Entity) -> Self::CustomIterator;
    fn get_forward_neighbors(&self, id: &Self::Entity) -> Self::CustomIterator;
    fn get_backward_neighbors(&self, id: &Self::Entity) -> Self::CustomIterator;
    fn get_neighbors(&self, id: &Self::Entity) -> Self::CustomIterator;
}
impl Querying for Arc<EngineState> {
    type Entity = EntityId;
    type CustomIterator = QueryIterator;

    fn get_edges(&self, id: &EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(id) {
            if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(id) {
                return (
                    self,
                    by_source
                        .elements()
                        .union(by_target.elements().to_owned())
                        .unique()
                        .into_iter()
                        .filter(|i| i != id)
                        .collect_vec(),
                )
                    .into();
            }
        }

        QueryIterator::default()
    }

    fn get_descriptors(&self, id: &EntityId) -> QueryIterator {
        if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(id) {
            (
                self,
                by_target
                    .elements()
                    .iter()
                    .filter(|&i| i != id)
                    .flat_map(|&i| self.get_brick(i))
                    .filter(|b| b.source == b.id)
                    .map(|b| b.id)
                    .collect_vec(),
            )
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn get_extensions(&self, id: &EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(id) {
            (
                self,
                by_source
                    .elements()
                    .iter()
                    .filter(|&i| i != id)
                    .flat_map(|&i| self.get_brick(i))
                    .filter(|b| b.target == b.id)
                    .map(|b| b.id)
                    .collect_vec(),
            )
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn get_properties(&self, id: &EntityId) -> QueryIterator {
        self.get_descriptors(id).union(self.get_extensions(id))
    }

    fn get_forward_neighbors(&self, id: &EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(id) {
            (
                self,
                by_source
                    .elements()
                    .iter()
                    .flat_map(|&i| self.get_brick(i))
                    .filter(|b| b.source != b.target && b.target != *id)
                    .map(|b| b.target)
                    .collect_vec(),
            )
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn get_backward_neighbors(&self, id: &EntityId) -> QueryIterator {
        if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(id) {
            (
                self,
                by_target
                    .elements()
                    .iter()
                    .flat_map(|&i| self.get_brick(i))
                    .filter(|b| b.source != b.target && b.source != *id)
                    .map(|b| b.source)
                    .collect_vec(),
            )
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn get_neighbors(&self, id: &EntityId) -> QueryIterator {
        self.get_forward_neighbors(id)
            .union(self.get_backward_neighbors(id))
    }
}

impl Querying for Arc<MosaicEngine> {
    type Entity = Tile;
    type CustomIterator = TileIterator;
    fn get_edges(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_edges(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }

    fn get_descriptors(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_descriptors(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }

    fn get_extensions(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_extensions(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }

    fn get_properties(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_properties(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }

    fn get_forward_neighbors(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_forward_neighbors(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }

    fn get_backward_neighbors(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_backward_neighbors(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }

    fn get_neighbors(&self, tile: &Tile) -> TileIterator {
        (
            self,
            self.engine_state
                .get_neighbors(&tile.id())
                .into_iter()
                .flat_map(|e| self.get_tile(*e))
                .collect_vec(),
        )
            .into()
    }
}

impl Querying for QueryIterator {
    type Entity = EntityId;
    type CustomIterator = QueryIterator;

    fn get_edges(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_edges(id)
    }

    fn get_descriptors(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_descriptors(id)
    }

    fn get_extensions(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_extensions(id)
    }

    fn get_properties(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_properties(id)
    }

    fn get_forward_neighbors(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_forward_neighbors(id)
    }

    fn get_backward_neighbors(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_backward_neighbors(id)
    }

    fn get_neighbors(&self, id: &EntityId) -> QueryIterator {
        self.engine.get_neighbors(id)
    }
}
impl Querying for TileIterator {
    type Entity = Tile;
    type CustomIterator = TileIterator;

    fn get_edges(&self, tile: &Tile) -> TileIterator {
        self.engine.get_edges(tile)
    }

    fn get_descriptors(&self, tile: &Tile) -> TileIterator {
        self.engine.get_descriptors(tile)
    }

    fn get_extensions(&self, tile: &Tile) -> TileIterator {
        self.engine.get_extensions(tile)
    }

    fn get_properties(&self, tile: &Tile) -> TileIterator {
        self.engine.get_properties(tile)
    }

    fn get_forward_neighbors(&self, tile: &Tile) -> TileIterator {
        self.engine.get_forward_neighbors(tile)
    }

    fn get_backward_neighbors(&self, tile: &Tile) -> TileIterator {
        self.engine.get_backward_neighbors(tile)
    }

    fn get_neighbors(&self, tile: &Tile) -> TileIterator {
        self.engine.get_neighbors(tile)
    }
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod querying_testing {
    use std::sync::Arc;

    use super::Querying;
    use crate::{
        internals::{lifecycle::Lifecycle, EngineState, EntityId},
        layers::{indirection::Indirection, parenting::Parenting},
    };

    #[test]
    fn test_query_edges() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ab = engine_state.create_arrow(&a, &b, "Arrow".into(), vec![]);
        let queried = engine_state.get_edges(&a);
        let mut query = queried.as_vec();
        query.sort();
        assert_eq!(vec![4], query);
    }

    #[test]
    fn test_neighbors_forward() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ab = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        let _bc = engine_state
            .create_arrow(&b, &c, "Arrow".into(), vec![])
            .unwrap();

        assert_eq!(vec![b], engine_state.get_forward_neighbors(&a).as_vec());
        assert_eq!(vec![c], engine_state.get_forward_neighbors(&b).as_vec());
        assert_eq!(0, engine_state.get_forward_neighbors(&c).len());
    }

    #[test]
    fn test_neighbors_backward() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ab = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        let _bc = engine_state
            .create_arrow(&b, &c, "Arrow".into(), vec![])
            .unwrap();

        assert_eq!(0, engine_state.get_backward_neighbors(&a).len());
        assert_eq!(vec![a], engine_state.get_backward_neighbors(&b).as_vec());
        assert_eq!(vec![b], engine_state.get_backward_neighbors(&c).as_vec());
    }

    #[test]
    fn test_neighbors() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _ab = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        let _bc = engine_state
            .create_arrow(&b, &c, "Arrow".into(), vec![])
            .unwrap();

        fn assert_neighbors(engine_state: &Arc<EngineState>, v: Vec<EntityId>, id: EntityId) {
            let mut w = v.clone();
            w.sort();

            let mut neighbors = engine_state.get_neighbors(&id).as_vec();
            neighbors.sort();

            assert_eq!(w, neighbors);
        }

        assert_neighbors(&engine_state, vec![b], a);
        assert_neighbors(&engine_state, vec![a, c], b);
        assert_neighbors(&engine_state, vec![b], c);
    }

    #[test]
    fn test_get_descriptors() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Property: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .add_descriptor(&a, "Property".into(), vec![])
            .unwrap();
        let d = engine_state
            .add_descriptor(&a, "Property".into(), vec![])
            .unwrap();
        let mut descriptors = engine_state.get_descriptors(&a);
        descriptors.sort();
        assert_eq!(vec![c, d], descriptors.as_vec());
    }

    #[test]
    fn test_get_extensions() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Property: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let _b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .add_outgoing_property(a, "Property".into(), vec![])
            .unwrap();
        let d = engine_state
            .add_outgoing_property(a, "Property".into(), vec![])
            .unwrap();
        let mut extensions = engine_state.get_extensions(&a);
        extensions.sort();
        assert_eq!(vec![c, d], extensions.as_vec());
    }

    /*
          e
    /-- A-->B
    |   ^   ^
    f   |   |
    |   parent
    |    \ /
    \---->C
          ^
          |
        parent
          |
          D
     */
    #[test]
    fn test_nested_query() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Parent: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let d = engine_state.create_object("Object".into(), vec![]).unwrap();

        let _p1 = engine_state.set_parent(&a, &c).unwrap();
        let _p2 = engine_state.set_parent(&b, &c).unwrap();
        let _p3 = engine_state.set_parent(&c, &d).unwrap();

        let _e = engine_state
            .create_arrow(&a, &b, "Arrow".into(), vec![])
            .unwrap();
        let f = engine_state
            .create_arrow(&a, &c, "Arrow".into(), vec![])
            .unwrap();
        println!(
            "{:?}",
            engine_state
                .entities_by_target_index
                .lock()
                .unwrap()
                .get(&a)
        );

        let edges = engine_state.get_edges(&c);
        assert_eq!(4, edges.len());

        let non_parent_edges = engine_state
            .build_query()
            .select_from(engine_state.get_edges(&c).as_vec())
            .without_component("Parent".into())
            .get();

        assert_eq!(vec![f], non_parent_edges.as_vec());
    }

    /*
       A<---B
       ^\----D
       |
       |
       parent
       |
       C

    */
    #[test]
    fn test_neighbor_filter_parent() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Parent: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state.create_object("Object".into(), vec![]).unwrap();
        let d = engine_state.create_object("Object".into(), vec![]).unwrap();

        let _p1 = engine_state.set_parent(&a, &c).unwrap();
        let _ba = engine_state
            .create_arrow(&b, &a, "Arrow".into(), vec![])
            .unwrap();
        let _da = engine_state
            .create_arrow(&d, &a, "Arrow".into(), vec![])
            .unwrap();

        let mut q1 = engine_state
            .build_query()
            .select_from(engine_state.get_edges(&a).as_vec())
            .without_component("Parent".into())
            .get_sources()
            .as_vec();

        q1.sort();

        let mut q2 = engine_state
            .get_edges(&a)
            .build_query()
            .without_component("Parent".into())
            .get_sources()
            .as_vec();
        q2.sort();

        assert_eq!(q1, q2);
        assert_eq!(vec![2, 4], q1);
    }
}
