use array_tool::vec::{Union, Uniq};
use itertools::Itertools;

use crate::internals::{query_iterator::QueryIterator, EngineState, EntityId};

pub trait Querying {
    fn query_edges(&self, id: EntityId) -> QueryIterator;
    fn query_descriptors(&self, id: EntityId) -> QueryIterator;
    fn query_extensions(&self, id: EntityId) -> QueryIterator;
    fn query_forward_neighbors(&self, id: EntityId) -> QueryIterator;
    fn query_backward_neighbors(&self, id: EntityId) -> QueryIterator;
    fn query_neighbors(&self, id: EntityId) -> QueryIterator;
}

impl Querying for EngineState {
    fn query_edges(&self, id: EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(&id) {
            if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(&id) {
                return by_source
                    .elements()
                    .union(by_target.elements().to_owned())
                    .unique()
                    .into_iter()
                    .filter(|i| i != &id)
                    .collect_vec()
                    .into();
            }
        }

        QueryIterator::default()
    }

    fn query_descriptors(&self, id: EntityId) -> QueryIterator {
        if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(&id) {
            by_target
                .elements()
                .into_iter()
                .filter(|&i| i != &id)
                .map(|&i| self.get_brick(i))
                .filter(|b| b.source == b.id)
                .map(|b| b.id)
                .collect_vec()
                .into()
        } else {
            QueryIterator::default()
        }
    }

    fn query_extensions(&self, id: EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(&id) {
            by_source
                .elements()
                .into_iter()
                .filter(|&i| i != &id)
                .map(|&i| self.get_brick(i))
                .filter(|b| b.target == b.id)
                .map(|b| b.id)
                .collect_vec()
                .into()
        } else {
            QueryIterator::default()
        }
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
    use crate::{
        internals::{EngineState, EntityId},
        layers::{indirection::Indirection, parenting::Parenting, traversing::Traversing},
    };

    #[test]
    fn test_query_edges() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void;");
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _c = engine_state.create_object_raw("Object".into(), vec![]);
        let _ab = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        let queried = engine_state.query_edges(a);
        let mut query = queried.as_vec();
        query.sort();
        assert_eq!(vec![4], query);
    }

    #[test]
    fn test_neighbors_forward() {
        let engine_state = EngineState::new();
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
        let engine_state = EngineState::new();
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
        let engine_state = EngineState::new();
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

    #[test]
    fn test_get_descriptors() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Property: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .add_incoming_property(a, "Property".into(), vec![])
            .unwrap();
        let d = engine_state
            .add_incoming_property(a, "Property".into(), vec![])
            .unwrap();
        let mut descriptors = engine_state.query_descriptors(a);
        descriptors.sort();
        assert_eq!(vec![c, d], descriptors.as_vec());
    }

    #[test]
    fn test_get_extensions() {
        let engine_state = EngineState::new();
        let _ = engine_state.add_component_types("Object: void; Arrow: void; Property: void;");
        let a = engine_state.create_object("Object".into(), vec![]).unwrap();
        let b = engine_state.create_object("Object".into(), vec![]).unwrap();
        let c = engine_state
            .add_outgoing_property(a, "Property".into(), vec![])
            .unwrap();
        let d = engine_state
            .add_outgoing_property(a, "Property".into(), vec![])
            .unwrap();
        let mut extensions = engine_state.query_extensions(a);
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

        let _p1 = engine_state.set_parent(a, c).unwrap();
        let _p2 = engine_state.set_parent(b, c).unwrap();
        let _p3 = engine_state.set_parent(c, d).unwrap();

        let _e = engine_state
            .create_arrow(a, b, "Arrow".into(), vec![])
            .unwrap();
        let f = engine_state
            .create_arrow(a, c, "Arrow".into(), vec![])
            .unwrap();
        println!(
            "{:?}",
            engine_state
                .entities_by_target_index
                .lock()
                .unwrap()
                .get(&a)
        );

        let edges = engine_state.query_edges(c);
        assert_eq!(4, edges.len());

        let non_parent_edges = engine_state
            .query()
            .select_from(engine_state.query_edges(c))
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

        let _p1 = engine_state.set_parent(a, c).unwrap();
        let _ba = engine_state
            .create_arrow(b, a, "Arrow".into(), vec![])
            .unwrap();
        let _da = engine_state
            .create_arrow(d, a, "Arrow".into(), vec![])
            .unwrap();

        println!("{:?}", engine_state.query_neighbors(a));
        println!(
            "{:?}",
            engine_state
                .query()
                .select_from(engine_state.query_edges(a))
                .without_component("Parent".into())
                .get_sources()
        );
        println!(
            "{:?}",
            engine_state
                .query()
                .select_from(engine_state.query_neighbors(a))
                .without_component("Parent".into())
                .get()
        );
    }
}
