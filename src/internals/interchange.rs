use super::datatypes::{S32, EntityId};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[derive(Debug)]
/// Bricks are the essential building blocks and hold a single component.
/// Every brick contains a single morphism and associated data
pub struct Brick {
    /// Identity of this element
    pub id: EntityId,
    /// The source element of this morphism
    pub source: EntityId,
    /// The target element of this morphism
    pub target: EntityId,
    /// The name of the component representing the data in this morphism
    pub component: S32,
    /// The actual data carried by the morphism
    pub data: Vec<u8>,
}

/* /////////////////////////////////////////////////////////////////////////////////// */
/// Unit Tests
/* /////////////////////////////////////////////////////////////////////////////////// */

#[cfg(test)]
mod interchange_testing {
    use std::{hash::{Hash, Hasher}, collections::hash_map::DefaultHasher};

    #[derive(Hash)]
    struct A {
        a: u8,
        b: u8,
        c: String,
    }

    #[test]
    fn hash_of_a() {
        let mut hasher = DefaultHasher::new();
        let a = A{ a: b'c', b: b'a', c: format!("qweqweijwqeiofjwioefjwoeifjoiwefjewf") };
        a.hash(&mut hasher);
        println!("{:?}", hasher.finish());
    }
}