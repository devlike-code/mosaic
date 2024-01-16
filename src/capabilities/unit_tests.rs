#[cfg(test)]
mod selection_tests {
    use itertools::Itertools;

    use crate::{
        capabilities::SelectionCapability,
        internals::{void, Mosaic, MosaicCRUD, MosaicIO},
    };

    #[test]
    fn test_selection() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", void());
        let b = mosaic.new_object("void", void());
        let c = mosaic.new_object("void", void());
        let ab = mosaic.new_arrow(&a, &b, "void", void());
        let _ac = mosaic.new_arrow(&a, &c, "void", void());
        let _bc = mosaic.new_arrow(&b, &c, "void", void());
        let s = mosaic.make_selection(&[a.clone(), b.clone(), ab]);

        assert_eq!(3, mosaic.get_selection(&s).len());
        let s = mosaic.make_selection(&[a.clone(), b]);
        assert_eq!(2, mosaic.get_selection(&s).len());
        let s = mosaic.make_selection(&[a]);
        assert_eq!(1, mosaic.get_selection(&s).len());
    }

    #[test]
    fn test_update_selection() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", void());
        let b = mosaic.new_object("void", void());
        let c = mosaic.new_object("void", void());
        let ab = mosaic.new_arrow(&a, &b, "void", void());
        let ac = mosaic.new_arrow(&a, &c, "void", void());
        let bc = mosaic.new_arrow(&b, &c, "void", void());

        let s = mosaic.make_selection(&[a.clone(), b.clone(), ab]);
        assert_eq!(
            vec![0, 1, 3],
            mosaic
                .get_selection(&s)
                .map(|t| t.id)
                .sorted()
                .collect_vec()
        );

        mosaic.update_selection(&s, &[a.clone(), b.clone(), ac, bc]);
        assert_eq!(
            vec![0, 1, 4, 5],
            mosaic
                .get_selection(&s)
                .map(|t| t.id)
                .sorted()
                .collect_vec()
        );
    }
}

#[cfg(test)]
mod archetype_tests {
    use crate::{
        capabilities::ArchetypeSubject,
        internals::{
            pars, void, ComponentValuesBuilderSetter, Mosaic, MosaicCRUD, MosaicIO,
            MosaicTypelevelCRUD, Value,
        },
    };

    #[test]
    fn test_archetypes() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Position: { x: f32, y: f32 };").unwrap();

        let a = mosaic.new_object("void", void());
        let p = a.add_component("Position", pars().set("x", 10.0f32).set("y", 6.0f32).ok());

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

    #[test]
    fn test_matching_archetypes() {
        let mosaic = Mosaic::new();
        mosaic.new_type("Position: { x: f32, y: f32 };").unwrap();
        mosaic.new_type("Label: s32;").unwrap();

        let a = mosaic.new_object("void", void());
        let p = a.add_component("Position", pars().set("x", 10.0f32).set("y", 6.0f32).ok());
        let l = a.add_component("Label", pars().set("self", "Hello world").ok());

        if a.match_archetype(&["Position", "Label"]) {
            let values = a.get_archetype(&["Position", "Label"]);
            let pos = values.get("Position").unwrap();
            let lab = values.get("Label").unwrap();

            assert_eq!(pos, &p);
            assert_eq!(lab, &l);
        }
    }
}

#[cfg(test)]
mod queue_tests {
    use itertools::Itertools;

    use crate::{
        capabilities::QueueCapability,
        internals::{void, Mosaic, MosaicIO},
        iterators::tile_getters::TileGetters,
    };

    #[test]
    fn test_queues() {
        let mosaic = Mosaic::new();
        let a = mosaic.new_object("void", void());
        let b = mosaic.new_object("void", void());
        let c = mosaic.new_object("void", void());

        let q = mosaic.make_queue();
        println!("{:?}: {:?}", q, q.iter().get_arrows().collect_vec());
        assert!(mosaic.is_queue_empty(&q));
        mosaic.enqueue(&q, &a);
        println!("{:?}: {:?}", q, q.iter().get_arrows().collect_vec());
        assert!(!mosaic.is_queue_empty(&q));
        mosaic.enqueue(&q, &b);
        println!("{:?}: {:?}", q, q.iter().get_arrows().collect_vec());
        mosaic.enqueue(&q, &c);
        println!("{:?}: {:?}", q, q.iter().get_arrows().collect_vec());

        assert_eq!(Some(a), mosaic.dequeue(&q));
        assert_eq!(Some(b), mosaic.dequeue(&q));
        assert_eq!(Some(c), mosaic.dequeue(&q));
        assert_eq!(None, mosaic.dequeue(&q));
    }
}
