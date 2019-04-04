use std::fmt::{Display, Formatter};
use std::io;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    code: StatusCode,
    message: String,
}

#[derive(Debug, Copy, Clone)]
pub enum StatusCode {
    Ok = 0,
    IOError = 1,
    UnknownAction = 2,
}

impl Error {
    pub fn new(code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
        write!(f, "{}: {}", self.code.to_string(), self.message)
    }
}

impl ToString for StatusCode {
    fn to_string(&self) -> String {
        unimplemented!()
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self {
            code: StatusCode::IOError,
            message: err.to_string(),
        }
    }
}

impl std::error::Error for Error {}
