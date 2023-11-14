use log::error;

pub(crate) fn report_error<A, S: AsRef<str>>(message: S) -> Result<A, String> {
    error!("{}", message.as_ref());
    Err(message.as_ref().to_owned())
}
