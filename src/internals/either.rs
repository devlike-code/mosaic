/// A structure denoting whether an entry had previously existed
/// in the map that's being indexed, or not.
pub enum EntryExistsResult<T> {
    Existed(T),
    Inserted(T),
}

impl<T: Clone> EntryExistsResult<T> {
    pub fn unwrap(&self) -> T {
        match self {
            EntryExistsResult::Existed(t) => t,
            EntryExistsResult::Inserted(t) => t,
        }
        .clone()
    }
}
