#[cfg(test)]
mod string_tests {
    use itertools::Itertools;

    use crate::{capabilities::StringFunnel, internals::Mosaic};

    #[test]
    fn test_string_funnel() {
        let mosaic = Mosaic::new();
        let hello_world = mosaic.create_string_object("hello world").unwrap();
        assert!(mosaic.string_exists("hello world"));
        assert!(hello_world.is_object());
        assert!(!hello_world
            .get_extensions_with(&mosaic)
            .collect_vec()
            .is_empty());
        assert_eq!(
            Some("hello world".to_string()),
            mosaic.get_string_value(&hello_world)
        );

        mosaic.delete_string("hello world");
        assert!(!mosaic.string_exists("hello world"));
    }
}

#[cfg(test)]
mod traversal_tests {

    use itertools::Itertools;

    use crate::{
        capabilities::{traversal::Traverse, Traversal},
        internals::{Mosaic, MosaicCRUD, MosaicTypelevelCRUD},
    };

    #[test]
    fn test_neighborhoods() {
        let t = Traversal::Exclude {
            components: &["Parent", "Child"],
        };

        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let c = mosaic.new_object("DEBUG");
        let d = mosaic.new_object("DEBUG");

        /*
                      /----> b
           a ----parent----> c
                      \----> d

           a ----> b <----> c -----> d
        */
        mosaic.new_arrow(&a, &b, "Parent");
        mosaic.new_arrow(&a, &c, "Parent");
        mosaic.new_arrow(&a, &d, "Parent");
        mosaic.new_arrow(&a, &b, "DEBUG");
        mosaic.new_arrow(&b, &c, "DEBUG");
        mosaic.new_arrow(&c, &b, "DEBUG");
        mosaic.new_arrow(&c, &d, "DEBUG");

        let p = mosaic.traverse(t);
        assert_eq!(1, p.out_degree(&a));
        assert_eq!(0, p.in_degree(&a));

        assert_eq!(1, p.out_degree(&b));
        assert_eq!(2, p.in_degree(&b));

        let a_fwd_neighbors = p.get_forward_neighbors(&a).collect_vec();
        assert!(a_fwd_neighbors.contains(&b));

        let a_bwd_neighbors = p.get_backward_neighbors(&a).collect_vec();
        assert!(a_bwd_neighbors.is_empty());

        assert_eq!(None, p.get_forward_neighbors(&d).next());

        let c_fwd_neighbors = p.get_forward_neighbors(&c).collect_vec();
        assert!(c_fwd_neighbors.contains(&b));
        assert!(c_fwd_neighbors.contains(&d));

        let c_bwd_neighbors = p.get_backward_neighbors(&c).collect_vec();
        assert!(c_bwd_neighbors.contains(&b));

        let d_bwd_neighbors = p.get_backward_neighbors(&d).collect_vec();
        assert!(d_bwd_neighbors.contains(&c));
    }

    #[test]
    fn test_dfs() {
        let t = Traversal::Exclude { components: &[] };

        let mosaic = Mosaic::new();
        let a = mosaic.new_object("DEBUG");
        let b = mosaic.new_object("DEBUG");
        let c = mosaic.new_object("DEBUG");
        let d = mosaic.new_object("DEBUG");
        let e = mosaic.new_object("DEBUG");

        /*
                      /----> b
           a ----parent----> c
                      \----> d

           4
           e ---------------|
           ^                v
           |       1        2        3
         0 a ----> b <----> c -----> d
                            2 -----> 3
                   1 <----- 2
                   1 -----> 2

        */
        mosaic.new_arrow(&a, &b, "DEBUG");
        mosaic.new_arrow(&e, &c, "DEBUG");
        mosaic.new_arrow(&a, &e, "DEBUG");
        mosaic.new_arrow(&b, &c, "DEBUG");
        mosaic.new_arrow(&c, &b, "DEBUG");
        mosaic.new_arrow(&c, &d, "DEBUG");

        let op = mosaic.traverse(t);

        println!("FORWARD FROM A: {:?}", op.get_forward_paths(&a));
        println!("FORWARD FROM C: {:?}", op.get_forward_paths(&c));
    }

    #[test]
    fn test_simple_reachability() {
        let mosaic = Mosaic::new();

        let _ = mosaic.new_type("Object: void; Arrow: void;");

        let a = mosaic.new_object("Object");
        let b = mosaic.new_object("Object");
        let d = mosaic.new_object("Object");
        let e = mosaic.new_object("Object");
        /*
            a -- x ---> b ----- y
                        |       |
                        |       |
                        v ----> d -- z --> e

        */
        let _x = mosaic.new_arrow(&a, &b, "Arrow");
        let y = mosaic.new_arrow(&b, &d, "Arrow");
        let v = mosaic.new_arrow(&b, &d, "Arrow");
        let _z = mosaic.new_arrow(&d, &e, "Arrow");

        let t = Traversal::Exclude { components: &[] };

        let op = mosaic.traverse(t);

        assert!(op.are_reachable(&a, &e));
        mosaic.delete_tile(v);
        assert!(op.are_reachable(&a, &e));
        mosaic.delete_tile(y);
        assert!(!op.are_reachable(&a, &e));
    }
}
