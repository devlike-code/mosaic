use std::{collections::HashMap, sync::Arc};

use itertools::Itertools;

use crate::internals::{
    query_iterator::QueryIterator, Block, DataBrick, EngineState, EntityId, Tile,
};

use super::{accessing::Accessing, querying::Querying};

pub(crate) fn tile_from_brick_data(engine: &Arc<EngineState>, brick: &DataBrick) -> Tile {
    (engine, brick).into()
}

pub trait Tiling {
    fn get_blocks(&self, selection: Option<QueryIterator>) -> HashMap<EntityId, Block>;
    fn get_tile(&self, brick: EntityId) -> Tile;
}

impl Tiling for Arc<EngineState> {
    fn get_blocks(&self, filter: Option<QueryIterator>) -> HashMap<EntityId, Block> {
        let selection = if filter.is_none() {
            self.query_access().get()
        } else {
            filter
                .unwrap()
                .as_slice()
                .into_iter()
                .fold(vec![].into(), |old: QueryIterator, &f| {
                    old.union(self.query_edges(f))
                })
        };

        let tiles = selection
            .as_slice()
            .into_iter()
            .map(|id| {
                let brick = self.get(*id).unwrap();
                let tile: Tile = (self, &brick).into();
                tile
            })
            .collect_vec();

        let mut result = HashMap::new();

        for tile in tiles {
            let id = tile.id();
            if !result.contains_key(&id) {
                result.insert(id, Block::new());
            }
            result
                .get_mut(&id)
                .unwrap()
                .extend(vec![tile.clone()].into());

            let (source, target) = tile.get_endpoints();
            if !result.contains_key(&source) {
                result.insert(source, Block::new());
            }
            result
                .get_mut(&source)
                .unwrap()
                .extend(vec![tile.clone()].into());

            if !result.contains_key(&target) {
                result.insert(target, Block::new());
            }
            result
                .get_mut(&target)
                .unwrap()
                .extend(vec![tile.polarize_towards(target)].into());
        }
        result
    }

    fn get_tile(&self, brick: EntityId) -> Tile {
        let brick = self.get_brick(brick);
        tile_from_brick_data(self, &brick)
    }
}

#[cfg(test)]
mod tiling_testing {
    use crate::{
        internals::{DatatypeValue, EngineState, Tile},
        layers::tiling::Tiling,
    };

    #[test]
    fn test_get_tiles() {
        let engine_state = EngineState::new();
        engine_state.add_component_types("Object: void; Arrow: void; Color: i32; Number: u32; Position: product { x: u32, y: u32 };").unwrap();
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let ab = engine_state
            .create_arrow(a, b, "Arrow".into(), vec![])
            .unwrap();
        let _pab = engine_state
            .add_incoming_property(ab, "Color".into(), vec![DatatypeValue::I32(0)])
            .unwrap();
        let _ba = engine_state
            .create_arrow(b, a, "Arrow".into(), vec![])
            .unwrap();
        let _pc = engine_state
            .add_incoming_property(a, "Color".into(), vec![DatatypeValue::I32(7)])
            .unwrap();
        let _pp = engine_state
            .add_incoming_property(
                a,
                "Position".into(),
                vec![DatatypeValue::U32(1), DatatypeValue::U32(2)],
            )
            .unwrap();
        let _pn = engine_state
            .add_incoming_property(b, "Number".into(), vec![DatatypeValue::U32(123)])
            .unwrap();
        let _en = engine_state
            .add_outgoing_property(
                b,
                "Position".into(),
                vec![DatatypeValue::U32(3), DatatypeValue::U32(4)],
            )
            .unwrap();
        let tiles: std::collections::HashMap<usize, crate::internals::Block> =
            engine_state.get_blocks(None);

        assert_eq!(5, tiles.get(&a).unwrap().tiles.len());
        assert_eq!(5, tiles.get(&b).unwrap().tiles.len());
        assert_eq!(2, tiles.get(&ab).unwrap().tiles.len());
    }

    #[test]
    fn test_get_tile() {
        let engine_state = EngineState::new();
        engine_state.add_component_types("Object: void; Arrow: void; Color: i32; Number: u32; Position: product { x: u32, y: u32 };").unwrap();
        let a = engine_state
            .create_object(
                "Position".into(),
                vec![DatatypeValue::U32(3), DatatypeValue::U32(4)],
            )
            .unwrap();
        let mut tile: Tile = engine_state.get_tile(a);

        let old_brick = engine_state.get_brick(a);
        println!("{:?}", old_brick.data);

        tile["x"] = DatatypeValue::U32(7);
        assert_eq!(DatatypeValue::U32(7), tile["x"]);
        let _ = tile.commit(&engine_state);

        //   /-- x ---/  /-- y ---/ ?
        let data: Vec<u8> = vec![0, 0, 0, 7, 0, 0, 0, 4];
        let new_brick = engine_state.get_brick(a);
        println!("{:?}", new_brick.data);
        assert_eq!(data, new_brick.data);
    }
}
