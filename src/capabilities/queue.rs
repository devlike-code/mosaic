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
    fn peek_queue(&self, q: &Tile) -> Option<Tile>;
}

pub trait PrivateQueueCapability {
    fn get_next_in_queue(&self, q: &Tile) -> Option<Tile>;
    fn get_prev_from_end_in_queue(&self, queue: &Tile) -> Option<Tile>;
    fn get_prev_from_queue(&self, stop: &Tile) -> Option<Tile>;
    fn get_sentinel_in_queue(&self, queue: &Tile) -> Tile;
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
        let end = self.get_sentinel_in_queue(queue);
        self.get_prev_from_queue(&end)
    }

    fn get_sentinel_in_queue(&self, queue: &Tile) -> Tile {
        queue.get_component("QueueSentinel").unwrap()
    }
}

pub type QueueTile = Tile;

impl QueueCapability for Arc<Mosaic> {
    fn make_queue(&self) -> Tile {
        self.new_type("Queue: unit;").unwrap();
        self.new_type("QueueSentinel: unit;").unwrap();
        self.new_type("Enqueued: unit;").unwrap();

        let queue = self.new_object("Queue", void());
        let sentinel = self.new_extension(&queue, "QueueSentinel", void());
        self.new_arrow(&queue, &sentinel, "Enqueued", void());
        assert_eq!(self.get_sentinel_in_queue(&queue), sentinel);
        queue
    }

    fn is_queue_empty(&self, q: &Tile) -> bool {
        if let Some(queue) = q.get_component("Queue") {
            let queue_end = Some(self.get_sentinel_in_queue(&queue));
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
                let old_enq_arrows = next.iter().get_arrows_into().include_component("Enqueued");

                self.new_arrow(&queue, v, "Enqueued", void());
                self.new_arrow(v, &next, "Enqueued", void());

                old_enq_arrows.delete();
            } else {
                panic!("No next element found in queue - tail potentially lost");
            }
        } else {
            panic!("No Queue found");
        }
    }

    fn dequeue(&self, q: &Tile) -> Option<Tile> {
        q.get_component("Queue").and_then(|queue| {
            let end = self.get_sentinel_in_queue(&queue);
            self.get_prev_from_queue(&end).and_then(|prev| {
                if prev != queue {
                    self.get_prev_from_queue(&prev).map(|before| {
                        prev.iter()
                            .get_arrows()
                            .include_component("Enqueued")
                            .delete();
                        self.new_arrow(&before, &end, "Enqueued", void());
                        prev
                    })
                } else {
                    None
                }
            })
        })
    }

    fn peek_queue(&self, q: &Tile) -> Option<Tile> {
        q.get_component("Queue").and_then(|queue| {
            let end = self.get_sentinel_in_queue(&queue);
            self.get_prev_from_queue(&end).and_then(|prev| {
                if prev != queue {
                    self.get_prev_from_queue(&prev).map(|_| prev)
                } else {
                    None
                }
            })
        })
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
        assert_eq!(end, &mosaic.get_sentinel_in_queue(&q));
    }

    #[test]
    fn test_enqueue() {
        let mosaic = Mosaic::new();

        let q = mosaic.make_queue();
        let q_arrows = q.iter().get_arrows().collect_vec();

        assert_eq!(1, q_arrows.len());
        let end = q_arrows.iter().map(|a| a.target()).unique().collect_vec();
        assert_eq!(1, end.len());
        let end = end.first().unwrap();

        let a = mosaic.new_object("void", void());

        mosaic.enqueue(&q, &a);
        let q_arrows = q.iter().get_arrows().collect_vec();
        assert_eq!(1, q_arrows.len());
        let ends_after_enqueue: HashSet<Tile> =
            HashSet::from_iter(q_arrows.iter().map(|a| a.target()));

        println!("{}", mosaic.dot("QUEUE_TEST"));
        assert!(ends_after_enqueue.contains(&a));
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
        let _ = mosaic.get_sentinel_in_queue(&q);

        let a = mosaic.new_object("void", void());
        mosaic.enqueue(&q, &a);

        let da = mosaic.dequeue(&q);
        assert!(da.is_some());
        assert_eq!(a, da.unwrap());
    }
}
