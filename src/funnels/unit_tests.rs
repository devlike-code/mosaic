#[cfg(test)]
mod test_funnels {
    use itertools::Itertools;

    use crate::{funnels::StringFunnel, internals::Mosaic};

    #[test]
    fn test_string_funnel() {
        let mosaic = Mosaic::new();
        let hello_world = mosaic.create_string_object("hello world").unwrap();
        assert!(hello_world.is_object());
        assert!(!hello_world
            .get_extensions_with(&mosaic)
            .collect_vec()
            .is_empty());
        assert_eq!(
            Some("hello world".to_string()),
            mosaic.get_string_value(&hello_world)
        );
    }
}
