use array_tool::vec::{Union, Uniq};
use itertools::Itertools;

use crate::internals::{BrickEditing, EngineState, EntityId};

use super::query_iterator::QueryIterator;

pub trait Querying {
    fn query_related(&self, iterator: EntityId) -> QueryIterator;
    fn query_forward_neighbors(&self, id: EntityId) -> QueryIterator;
    fn query_backward_neighbors(&self, id: EntityId) -> QueryIterator;
    fn query_neighbors(&self, id: EntityId) -> QueryIterator;
}

impl Querying for EngineState {
    fn query_related(&self, id: EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(&id) {
            if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(&id) {
                return by_source
                    .elements()
                    .union(by_target.elements().to_owned())
                    .unique()
                    .into();
            }
        }

        QueryIterator::default()
    }

    fn query_forward_neighbors(&self, id: EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(&id) {
            by_source
                .elements()
                .into_iter()
                .map(|&i| self.get_brick(i))
                .filter(|b| b.source != b.target && b.target != id)
                .map(|b| b.target)
                .collect_vec()
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn query_backward_neighbors(&self, id: EntityId) -> QueryIterator {
        if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(&id) {
            by_target
                .elements()
                .into_iter()
                .map(|&i| self.get_brick(i))
                .filter(|b| b.source != b.target && b.source != id)
                .map(|b| b.source)
                .collect_vec()
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn query_neighbors(&self, id: EntityId) -> QueryIterator {
        self.query_forward_neighbors(id)
            .union(self.query_backward_neighbors(id))
    }
}

#[cfg(test)]
mod querying_testing {
    use super::Querying;
    use crate::internals::{EngineState, EntityId};

    #[test]
    fn test_query_related() {
        let engine_state = EngineState::default();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _c = engine_state.create_object_raw("Object".into(), vec![]);
        let _ab = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        let queried = engine_state.query_related(a);
        let mut query = queried.as_vec();
        query.sort();
        assert_eq!(vec![1, 4], query);
    }

    #[test]
    fn test_neighbors_forward() {
        let engine_state = EngineState::default();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let c = engine_state.create_object_raw("Object".into(), vec![]);
        let _ab = engine_state
            .create_arrow(a, b, "Arrow".into(), vec![])
            .unwrap();
        let _bc = engine_state
            .create_arrow(b, c, "Arrow".into(), vec![])
            .unwrap();

        assert_eq!(vec![b], engine_state.query_forward_neighbors(a).as_vec());
        assert_eq!(vec![c], engine_state.query_forward_neighbors(b).as_vec());
        assert_eq!(0, engine_state.query_forward_neighbors(c).len());
    }

    #[test]
    fn test_neighbors_backward() {
        let engine_state = EngineState::default();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let c = engine_state.create_object_raw("Object".into(), vec![]);
        let _ab = engine_state
            .create_arrow(a, b, "Arrow".into(), vec![])
            .unwrap();
        let _bc = engine_state
            .create_arrow(b, c, "Arrow".into(), vec![])
            .unwrap();

        assert_eq!(0, engine_state.query_backward_neighbors(a).len());
        assert_eq!(vec![a], engine_state.query_backward_neighbors(b).as_vec());
        assert_eq!(vec![b], engine_state.query_backward_neighbors(c).as_vec());
    }

    #[test]
    fn test_neighbors() {
        let engine_state = EngineState::default();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let c = engine_state.create_object_raw("Object".into(), vec![]);
        let _ab = engine_state
            .create_arrow(a, b, "Arrow".into(), vec![])
            .unwrap();
        let _bc = engine_state
            .create_arrow(b, c, "Arrow".into(), vec![])
            .unwrap();

        fn assert_neighbors(engine_state: &EngineState, v: Vec<EntityId>, id: EntityId) {
            let mut w = v.clone();
            w.sort();

            let mut neighbors = engine_state.query_neighbors(id).as_vec();
            neighbors.sort();

            assert_eq!(w, neighbors);
        }

        assert_neighbors(&engine_state, vec![b], a);
        assert_neighbors(&engine_state, vec![a, c], b);
        assert_neighbors(&engine_state, vec![b], c);
    }
}
