use array_tool::vec::{Union, Uniq};

use crate::internals::{EngineState, EntityId};

use super::query_iterator::QueryIterator;

pub trait Querying {
    fn query_related(&self, iterator: EntityId) -> QueryIterator;
}

impl Querying for EngineState {
    fn query_related(&self, id: EntityId) -> QueryIterator {
        let by_source = self.entities_by_source_index.lock().unwrap().get(&id).unwrap().elements().to_owned();
        let by_target = self.entities_by_target_index.lock().unwrap().get(&id).unwrap().elements().to_owned();
        by_source.union(by_target).unique().into()
    }
}

#[cfg(test)]
mod querying_testing {
    use crate::internals::EngineState;
    use super::Querying;

    #[test]
    fn test_query_all() {
        let engine_state = EngineState::default();
        let _ = engine_state.add_component_types("Arrow: void;");
        let a = engine_state.create_object_raw("Object".into(), vec![]);
        let b = engine_state.create_object_raw("Object".into(), vec![]);
        let _c = engine_state.create_object_raw("Object".into(), vec![]);
        let _ab = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        println!("{:?}", engine_state.query_related(a));
    }
}