use array_tool::vec::{Union, Uniq};

use crate::internals::{EngineState, EntityId};

use super::query_iterator::QueryIterator;

pub trait Querying {
    fn query_related(&self, iterator: EntityId) -> QueryIterator;
}

impl Querying for EngineState {
    fn query_related(&self, id: EntityId) -> QueryIterator {
        if let Some(by_source) = self.entities_by_source_index.lock().unwrap().get(&id) {
            if let Some(by_target) = self.entities_by_target_index.lock().unwrap().get(&id) {
                return by_source.elements().union(by_target.elements().to_owned()).unique().into();
            }
        }

        QueryIterator::default()
    }
}

#[cfg(test)]
mod querying_testing {
    use super::Querying;
    use crate::internals::EngineState;

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
