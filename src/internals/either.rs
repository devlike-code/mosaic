pub enum EitherByAge<T> {
    Old(T),
    New(T),
}

impl<T: Clone> EitherByAge<T> {
    pub fn unwrap(&self) -> T {
        match self {
            EitherByAge::Old(t) => t,
            EitherByAge::New(t) => t,
        }
        .clone()
    }
}
