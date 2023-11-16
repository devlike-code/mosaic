#[derive(Debug, Default, PartialEq, Clone)]
pub enum Traversal {
    #[default]
    Forward,
    Backward,
    Both,
}

pub trait Traversing {
    fn out_degree(&self, tile: &Tile) -> usize;
    fn in_degree(&self, tile: &Tile) -> usize;

    // fn depth_first_search(&self, src: &Self::Entity, traversal: Traversal) -> Vec<QueryIterator>;
    // fn reach_forward(&self, src: &Self::Entity) -> Vec<QueryIterator>;
    // fn reach_backward(&self, src: &Self::Entity) -> Vec<QueryIterator>;
    // fn reach_forward_to(&self, src: &Self::Entity, tgt: &Self::Entity) -> Option<QueryIterator>;
    // fn reach_backward_to(&self, src: &Self::Entity, tgt: &Self::Entity) -> Option<QueryIterator>;
    // fn are_reachable(&self, src: &Self::Entity, tgt: &Self::Entity) -> bool;
}

