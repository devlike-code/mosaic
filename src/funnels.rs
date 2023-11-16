pub mod parent_funnel;
pub mod string_funnel;

pub use parent_funnel::*;
pub use string_funnel::*;

#[cfg(test)]
mod test_funnels {
    use itertools::Itertools;

    use crate::internals::Mosaic;

    use super::StringFunnel;

    #[test]
    fn test_string_funnel() {
        let mosaic = Mosaic::new();
        mosaic
            .entity_registry
            .add_component_types("String: b128;")
            .unwrap();
        let hello_world = mosaic.create_string_object("hello world").unwrap();
        assert!(hello_world.is_object());
        assert!(!hello_world
            .get_extensions_with(&mosaic)
            .collect_vec()
            .is_empty());
    }
}
