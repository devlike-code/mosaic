use super::{Tile, ToByteArray, Value, S32};

pub trait TileFieldSetter<T: ToByteArray> {
    fn set(&mut self, index: &str, value: T);
}

impl TileFieldSetter<i8> for Tile {
    fn set(&mut self, index: &str, value: i8) {
        self.set_field(index, Value::I8(value))
    }
}

impl TileFieldSetter<i16> for Tile {
    fn set(&mut self, index: &str, value: i16) {
        self.set_field(index, Value::I16(value))
    }
}

impl TileFieldSetter<i32> for Tile {
    fn set(&mut self, index: &str, value: i32) {
        self.set_field(index, Value::I32(value))
    }
}

impl TileFieldSetter<i64> for Tile {
    fn set(&mut self, index: &str, value: i64) {
        self.set_field(index, Value::I64(value))
    }
}

impl TileFieldSetter<u8> for Tile {
    fn set(&mut self, index: &str, value: u8) {
        self.set_field(index, Value::U8(value))
    }
}

impl TileFieldSetter<u16> for Tile {
    fn set(&mut self, index: &str, value: u16) {
        self.set_field(index, Value::U16(value))
    }
}

impl TileFieldSetter<u32> for Tile {
    fn set(&mut self, index: &str, value: u32) {
        self.set_field(index, Value::U32(value))
    }
}

impl TileFieldSetter<u64> for Tile {
    fn set(&mut self, index: &str, value: u64) {
        self.set_field(index, Value::U64(value))
    }
}

impl TileFieldSetter<f32> for Tile {
    fn set(&mut self, index: &str, value: f32) {
        self.set_field(index, Value::F32(value))
    }
}

impl TileFieldSetter<f64> for Tile {
    fn set(&mut self, index: &str, value: f64) {
        self.set_field(index, Value::F64(value))
    }
}

impl TileFieldSetter<S32> for Tile {
    fn set(&mut self, index: &str, value: S32) {
        self.set_field(index, Value::S32(value))
    }
}

impl TileFieldSetter<String> for Tile {
    fn set(&mut self, index: &str, value: String) {
        self.set_field(index, Value::STR(value))
    }
}

impl TileFieldSetter<bool> for Tile {
    fn set(&mut self, index: &str, value: bool) {
        self.set_field(index, Value::BOOL(value))
    }
}

pub trait TileFieldEmptyQuery {
    type Output;

    fn query(&self) -> Self::Output;
}

pub trait TileFieldQuery<T> {
    type Output;

    fn get_by(&self, index: T) -> Self::Output;
}

impl TileFieldQuery<(&str, &str)> for Tile {
    type Output = (Value, Value);

    fn get_by(&self, index: (&str, &str)) -> Self::Output {
        let a = self.get(index.0);
        let b = self.get(index.1);
        (a, b)
    }
}

impl TileFieldQuery<(&str, &str, &str, &str)> for Tile {
    type Output = (Value, Value, Value, Value);

    fn get_by(&self, index: (&str, &str, &str, &str)) -> Self::Output {
        let a = self.get(index.0);
        let b = self.get(index.1);
        let c = self.get(index.2);
        let d = self.get(index.3);
        (a, b, c, d)
    }
}
