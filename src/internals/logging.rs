use anyhow::anyhow;

pub fn init_logging() {
    env_logger::init();
}

pub trait Logging {
    fn to_error<T>(self) -> anyhow::Result<T>;
}

impl<'a> Logging for &'a str {
    fn to_error<T>(self) -> anyhow::Result<T> {
        Err(anyhow!(self.to_string()))
    }
}

impl Logging for String {
    fn to_error<T>(self) -> anyhow::Result<T> {
        Err(anyhow!(self))
    }
}
