use std::sync::Arc;

use crate::{
    internals::{void, Mosaic, MosaicCRUD, MosaicIO, MosaicTypelevelCRUD, Tile},
    iterators::{
        component_selectors::ComponentSelectors, tile_deletion::TileDeletion,
        tile_getters::TileGetters,
    },
};

use super::ArchetypeSubject;

pub trait QueueCapability {
    fn make_queue(&self) -> Tile;
    fn is_queue_empty(&self, q: &Tile) -> bool;
    fn enqueue(&self, q: &Tile, v: &Tile);
    fn dequeue(&self, q: &Tile) -> Option<Tile>;
}

pub trait PrivateQueueCapability {
    fn get_next_in_queue(&self, q: &Tile) -> Option<Tile>;
    fn get_prev_from_end_in_queue(&self, queue: &Tile) -> Option<Tile>;
    fn get_prev_from_queue(&self, stop: &Tile) -> Option<Tile>;
    fn get_arrow_to_end_in_queue(&self, queue: &Tile) -> Tile;
    fn get_end_in_queue(&self, queue: &Tile) -> Tile;
}

impl PrivateQueueCapability for Arc<Mosaic> {
    fn get_next_in_queue(&self, queue: &Tile) -> Option<Tile> {
        queue
            .iter()
            .get_arrows_from()
            .include_component("Enqueued")
            .get_targets()
            .next()
    }

    fn get_prev_from_queue(&self, stop: &Tile) -> Option<Tile> {
        stop.iter()
            .get_arrows_into()
            .include_component("Enqueued")
            .get_sources()
            .next()
    }

    fn get_prev_from_end_in_queue(&self, queue: &Tile) -> Option<Tile> {
        let end = self.get_end_in_queue(queue);
        self.get_prev_from_queue(&end)
    }

    fn get_arrow_to_end_in_queue(&self, queue: &Tile) -> Tile {
        queue
            .iter()
            .get_arrows_from()
            .include_component("ToQueueHead")
            .next()
            .unwrap()
    }

    fn get_end_in_queue(&self, queue: &Tile) -> Tile {
        self.get_arrow_to_end_in_queue(queue).target()
    }
}

impl QueueCapability for Arc<Mosaic> {
    fn make_queue(&self) -> Tile {
        self.new_type("Queue: void;").unwrap();
        self.new_type("QueueHead: void;").unwrap();
        self.new_type("ToQueueHead: void;").unwrap();
        self.new_type("Enqueued: void;").unwrap();

        let q = self.new_object("Queue", void());
        let h = self.new_object("QueueHead", void());
        self.new_arrow(&q, &h, "ToQueueHead", void());
        self.new_arrow(&q, &h, "Enqueued", void());
        assert_eq!(self.get_end_in_queue(&q), h);
        q
    }

    fn is_queue_empty(&self, q: &Tile) -> bool {
        if let Some(queue) = q.get_component("Queue") {
            let queue_end = Some(self.get_end_in_queue(&queue));
            let enqueued = self.get_next_in_queue(&queue);

            println!("{:?} {:?}", queue_end, enqueued);
            queue_end == enqueued
        } else {
            false
        }
    }

    fn enqueue(&self, q: &Tile, v: &Tile) {
        if let Some(queue) = q.get_component("Queue") {
            if let Some(next) = self.get_next_in_queue(q) {
                let old_arrows = next.iter().get_arrows_into().include_component("Enqueued");

                self.new_arrow(&queue, v, "Enqueued", void());
                self.new_arrow(v, &next, "Enqueued", void());

                old_arrows.delete();
            }
        }
    }

    fn dequeue(&self, q: &Tile) -> Option<Tile> {
        if let Some(queue) = q.get_component("Queue") {
            let end = self.get_end_in_queue(&queue);
            match self.get_prev_from_queue(&end) {
                Some(prev) if prev != queue => {
                    if let Some(before) = self.get_prev_from_queue(&prev) {
                        prev.iter()
                            .get_arrows()
                            .include_component("Enqueued")
                            .delete();
                        self.new_arrow(&before, &end, "Enqueued", void());
                        Some(prev)
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod queue_unit_tests {
    use std::collections::HashSet;

    use itertools::Itertools;

    use crate::{
        capabilities::PrivateQueueCapability,
        internals::{void, Mosaic, MosaicIO, Tile},
        iterators::tile_getters::TileGetters,
    };

    use super::QueueCapability;

    #[test]
    fn test_dequeue_empty() {
        let mosaic = Mosaic::new();

        let q = mosaic.make_queue();
        assert_eq!(None, mosaic.dequeue(&q));
    }

    #[test]
    fn test_get_end_in_queue() {
        let mosaic = Mosaic::new();

        let q = mosaic.make_queue();
        let end = q
            .iter()
            .get_arrows()
            .map(|a| a.target())
            .unique()
            .collect_vec();
        assert_eq!(1, end.len());

        let end = end.first().unwrap();
        assert_eq!(end, &mosaic.get_end_in_queue(&q));
    }

    #[test]
    fn test_enqueue() {
        let mosaic = Mosaic::new();

        let q = mosaic.make_queue();
        let q_arrows = q.iter().get_arrows().collect_vec();

        assert_eq!(2, q_arrows.len());
        let end = q_arrows.iter().map(|a| a.target()).unique().collect_vec();
        assert_eq!(1, end.len());
        let end = end.first().unwrap();

        let a = mosaic.new_object("void", void());

        mosaic.enqueue(&q, &a);
        let q_arrows = q.iter().get_arrows().collect_vec();
        assert_eq!(2, q_arrows.len());
        let ends_after_enqueue: HashSet<Tile> =
            HashSet::from_iter(q_arrows.iter().map(|a| a.target()));

        assert!(ends_after_enqueue.contains(end));
        assert!(ends_after_enqueue.contains(&a));

        println!("ARROWS (Q): {:?}", q.iter().get_arrows());
        println!("ARROWS (A): {:?}", a.iter().get_arrows());
        println!("ARROWS (E): {:?}", end.iter().get_arrows());
    }

    #[test]
    fn test_prev_from_end_in_empty_queue() {
        let mosaic = Mosaic::new();

        let q = mosaic.make_queue();
        let p = mosaic.get_prev_from_end_in_queue(&q);
        assert_eq!(q, p.unwrap());
    }

    #[test]
    fn test_dequeue() {
        let mosaic = Mosaic::new();

        let q = mosaic.make_queue();
        let _ = mosaic.get_end_in_queue(&q);

        let a = mosaic.new_object("void", void());
        mosaic.enqueue(&q, &a);

        let da = mosaic.dequeue(&q);
        assert!(da.is_some());
        assert_eq!(a, da.unwrap());
    }
}
