use crate::internals::EngineState;

use super::query_iterator::QueryIterator;

pub struct Query {}

pub trait Querying {
    fn query_all(&self) -> Query;
    fn query_in(&self, iterator: QueryIterator) -> Query;
}

impl Querying for EngineState {
    fn query_all(&self) -> Query {
        Query{}
    }

    fn query_in(&self, _iterator: QueryIterator) -> Query {
        Query{}
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
        let a = engine_state.create_object();
        let b = engine_state.create_object();
        let _c = engine_state.create_object();
        let _ab = engine_state.create_arrow(a, b, "Arrow".into(), vec![]);
        engine_state.query_all();
    }
}