#[cfg(test)]
mod string_tests {
    use itertools::Itertools;

    use crate::{
        capabilities::StringCapability, internals::Mosaic, iterators::tile_getters::TileGetters,
    };

    #[test]
    fn test_string_funnel() {
        let mosaic = Mosaic::new();
        let hello_world = mosaic.create_string_object("hello world").unwrap();
        assert!(mosaic.string_exists("hello world"));
        assert!(hello_world.is_object());
        assert!(!hello_world
            .clone()
            .into_iter()
            .get_extensions()
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
        internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile},
    };

    #[test]
    fn test_neighborhoods() {
        let t = Traversal::Exclude {
            components: &["GroupOwner".to_string(), "Group".to_string()],
        };

        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", default_vals());
        let b = mosaic.new_object("void", default_vals());
        let c = mosaic.new_object("void", default_vals());
        let d = mosaic.new_object("void", default_vals());

        /*
                      /----> b
           a ----group ----> c
                      \----> d

           a ----> b <----> c -----> d
        */
        mosaic.new_arrow(&a, &b, "GroupOwner", default_vals());
        mosaic.new_arrow(&a, &c, "GroupOwner", default_vals());
        mosaic.new_arrow(&a, &d, "GroupOwner", default_vals());
        mosaic.new_arrow(&a, &b, "void", default_vals());
        mosaic.new_arrow(&b, &c, "void", default_vals());
        mosaic.new_arrow(&c, &b, "void", default_vals());
        mosaic.new_arrow(&c, &d, "void", default_vals());

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
        fn stringify_paths(paths: Vec<Vec<Tile>>) -> Vec<String> {
            paths.into_iter().map(stringify_path).collect_vec()
        }

        fn stringify_path(path: Vec<Tile>) -> String {
            path.into_iter().map(|t| format!("{}", t)).join("-")
        }

        let t = Traversal::Exclude { components: &[] };

        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", default_vals());
        let b = mosaic.new_object("void", default_vals());
        let c = mosaic.new_object("void", default_vals());
        let d = mosaic.new_object("void", default_vals());
        let e = mosaic.new_object("void", default_vals());

        /*
                      /----> b
           a ----group ----> c
                      \----> d

           4
           e ---------------|
           ^                v
           |       1        2        3
         0 a ----> b <----> c -----> d

                            2 -----> 3

                   1 <----- 2
                   1 -----> x

        */
        mosaic.new_arrow(&a, &b, "void", default_vals());
        mosaic.new_arrow(&e, &c, "void", default_vals());
        mosaic.new_arrow(&a, &e, "void", default_vals());
        mosaic.new_arrow(&b, &c, "void", default_vals());
        mosaic.new_arrow(&c, &b, "void", default_vals());
        mosaic.new_arrow(&c, &d, "void", default_vals());

        let op = mosaic.traverse(t);

        let paths_from_a = stringify_paths(op.get_forward_paths(&a));
        let paths_from_c = stringify_paths(op.get_forward_paths(&c));

        assert_eq!(3, paths_from_a.len());
        println!("{:?}", paths_from_a);
        assert!(paths_from_a.contains(&"(x|0)-(x|1)-(x|2)-(x|3)".to_string()));
        assert!(paths_from_a.contains(&"(x|0)-(x|4)-(x|2)-(x|3)".to_string()));
        assert!(paths_from_a.contains(&"(x|0)-(x|4)-(x|2)-(x|1)".to_string()));

        assert_eq!(2, paths_from_c.len());
        assert!(paths_from_c.contains(&"(x|2)-(x|1)".to_string()));
        assert!(paths_from_c.contains(&"(x|2)-(x|3)".to_string()));
    }

    #[test]
    fn test_simple_reachability() {
        let mosaic = Mosaic::new();

        let _ = mosaic.new_type("Object: unit; Arrow: unit;");

        let a = mosaic.new_object("Object", default_vals());
        let b = mosaic.new_object("Object", default_vals());
        let d = mosaic.new_object("Object", default_vals());
        let e = mosaic.new_object("Object", default_vals());
        /*
            a -- x ---> b ----- y
                        |       |
                        |       |
                        v ----> d -- z --> e

        */
        let _x = mosaic.new_arrow(&a, &b, "Arrow", default_vals());
        let y = mosaic.new_arrow(&b, &d, "Arrow", default_vals());
        let v = mosaic.new_arrow(&b, &d, "Arrow", default_vals());
        let _z = mosaic.new_arrow(&d, &e, "Arrow", default_vals());

        let t = Traversal::Exclude { components: &[] };

        let op = mosaic.traverse(t);

        assert!(op.forward_path_exists_between(&a, &e));
        mosaic.delete_tile(v);
        assert!(op.forward_path_exists_between(&a, &e));
        mosaic.delete_tile(y);
        assert!(!op.forward_path_exists_between(&a, &e));
    }

    #[test]
    fn test_limited_traversal() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", default_vals()); // 0
        let b = mosaic.new_object("void", default_vals()); // 1
        let c = mosaic.new_object("void", default_vals());
        let d = mosaic.new_object("void", default_vals());
        let e = mosaic.new_object("void", default_vals());

        let _ab = mosaic.new_arrow(&a, &b, "void", default_vals());
        let _ec = mosaic.new_arrow(&e, &c, "void", default_vals());
        let _ae = mosaic.new_arrow(&a, &e, "void", default_vals());
        let _bc = mosaic.new_arrow(&b, &c, "void", default_vals());
        let _cb = mosaic.new_arrow(&c, &b, "void", default_vals());
        let _cd = mosaic.new_arrow(&c, &d, "void", default_vals());

        let traversal = Traversal::Limited {
            tiles: vec![a.clone(), b.clone()],
            include_arrows: true,
        };

        let op = mosaic.traverse(traversal);

        println!("{:?}", op.out_degree(&a));
        println!("{:?}", op.in_degree(&a));

        println!("{:?}", op.out_degree(&b));
        println!("{:?}", op.in_degree(&b));

        println!("{:?}", op.get_arrows_from(&a));
        println!("{:?}", op.get_arrows_into(&b));
    }
}

#[cfg(test)]
mod grouping_tests {

    use crate::{
        capabilities::GroupingCapability,
        internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD},
    };

    #[test]
    fn group_owner_test() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Group: s32;").unwrap();

        let o = mosaic.new_object("void", default_vals());
        let b = mosaic.new_object("void", default_vals());
        let c = mosaic.new_object("void", default_vals());
        let d = mosaic.new_object("void", default_vals());

        /*
                         /----> b
           o ----group(p) ----> c
                         \----> d

        */

        mosaic.group("Parent", &o, &[b.clone(), c.clone(), d.clone()]);
        let e = mosaic.get_group_owner_descriptor("Parent", &o).unwrap();

        mosaic.group("Parent2", &o, &[b.clone(), c.clone(), d.clone()]);
        mosaic.group("Parent", &o, &[b.clone(), d.clone()]);

        let _p = mosaic.get_group_owner_descriptor("Parent", &b);
        assert!(!mosaic.is_tile_valid(&e));

        let c_memberships = mosaic.get_group_memberships(&c);
        assert!(c_memberships.len() == 1);
        assert_eq!(
            c_memberships.first().unwrap().get("self").as_s32(),
            "Parent2".into()
        );

        assert_eq!(mosaic.get_group_owner("Parent", &b), Some(o));
    }
}

#[cfg(test)]
mod process_tests {
    use std::sync::Arc;

    use itertools::Itertools;

    use crate::{
        capabilities::{process::ProcessCapability, GroupingCapability},
        internals::{
            self_val, Logging, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile, Value,
        },
        iterators::tile_getters::TileGetters,
    };

    #[test]
    fn test_processes() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Number: u32;").unwrap();

        let add = mosaic.create_process("add", &["a", "b"]).unwrap();
        let x = mosaic.new_object("Number", self_val(Value::U32(7)));
        let y = mosaic.new_object("Number", self_val(Value::U32(5)));

        mosaic.pass_process_parameter(&add, "a", &x).unwrap();
        mosaic.pass_process_parameter(&add, "b", &y).unwrap();

        fn do_add(mosaic: &Arc<Mosaic>, add_instance: &Tile) -> anyhow::Result<u32> {
            let args = mosaic.get_process_parameter_values(add_instance)?;
            let a = args.get(&"a".into()).unwrap();
            let b = args.get(&"b".into()).unwrap();

            match (&a, &b) {
                (Some(a), Some(b)) => Ok(a.get("self").as_u32() + b.get("self").as_u32()),
                _ => "Can't do add :(".to_error(),
            }
        }

        println!("{:?}", mosaic.get_group_owner_descriptor("add", &add));
        println!("{:?}", mosaic.get_group_members("add", &add).collect_vec());
        println!(
            "{:?}",
            mosaic
                .get_group_members("add", &add)
                .get_arrows_from()
                .collect_vec()
        );
        assert_eq!(12, do_add(&mosaic, &add).unwrap());

        mosaic.delete_tile(add);
        for i in 0..=5 {
            assert!(!mosaic.is_tile_valid(&i));
        }

        assert!(mosaic.is_tile_valid(&6));
        assert!(mosaic.is_tile_valid(&7));

        assert!(!mosaic.is_tile_valid(&8));
        assert!(!mosaic.is_tile_valid(&9));
    }
}

#[cfg(test)]
mod selection_tests {
    use crate::{
        capabilities::SelectionCapability,
        internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO},
    };

    #[test]
    fn test_selection() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", default_vals());
        let b = mosaic.new_object("void", default_vals());
        let c = mosaic.new_object("void", default_vals());
        let ab = mosaic.new_arrow(&a, &b, "void", default_vals());
        let _ac = mosaic.new_arrow(&a, &c, "void", default_vals());
        let _bc = mosaic.new_arrow(&b, &c, "void", default_vals());
        let s = mosaic.make_selection();
        mosaic.fill_selection(&s, &[a.clone(), b.clone(), ab]);
        assert_eq!(3, mosaic.get_selection(&s).len());
        mosaic.fill_selection(&s, &[a.clone(), b]);
        assert_eq!(2, mosaic.get_selection(&s).len());
        mosaic.fill_selection(&s, &[a]);
        assert_eq!(1, mosaic.get_selection(&s).len());
    }
}

#[cfg(test)]
mod archetype_tests {
    use crate::{
        capabilities::ArchetypeSubject,
        internals::{default_vals, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Value},
    };

    #[test]
    fn test_archetypes() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Position: { x: f32, y: f32 };").unwrap();

        let a = mosaic.new_object("void", default_vals());
        let p = a.add_component(
            "Position",
            vec![
                ("x".into(), Value::F32(10.0)),
                ("y".into(), Value::F32(6.0)),
            ],
        );

        assert!(mosaic.is_tile_valid(&p));
        assert!(p.is_descriptor());
        assert_eq!(p.target_id(), a.id);
        assert_eq!(p.component, "Position".into());
        assert_eq!(p.get("x"), Value::F32(10.0));
        assert_eq!(p.get("y"), Value::F32(6.0));

        let r = a.get_component("Position");
        assert!(r.is_some());
        let r = r.unwrap();
        assert_eq!(r, p);

        a.remove_component("Position");
        let q = a.get_component("Position");
        assert!(!mosaic.is_tile_valid(&p));
        assert!(q.is_none());
    }
}
