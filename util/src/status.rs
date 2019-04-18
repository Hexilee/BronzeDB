use std::fmt::{Display, Formatter};
use std::io;
use std::sync::PoisonError;
use std::u8::MAX;

#[derive(Debug, Copy, Clone, PartialOrd, PartialEq)]
pub enum StatusCode {
    OK = 0,
    IOError = 1,
    UnknownAction = 2,
    PoisonError = 3,
    NotFound = 4,
    UnknownStatusCode = MAX as isize,
}

impl From<u8> for StatusCode {
    fn from(code: u8) -> Self {
        match code {
            0 => StatusCode::OK,
            1 => StatusCode::IOError,
            2 => StatusCode::UnknownAction,
            3 => StatusCode::PoisonError,
            4 => StatusCode::NotFound,
            _ => StatusCode::UnknownStatusCode,
        }
    }
}

impl ToString for StatusCode {
    fn to_string(&self) -> String {
        match self {
            StatusCode::OK => "OK".into(),
            StatusCode::IOError => "IOError".into(),
            StatusCode::UnknownAction => "UnknownAction".into(),
            StatusCode::PoisonError => "PoisonError".into(),
            StatusCode::NotFound => "NotFound".into(),
            StatusCode::UnknownStatusCode => "UnknownStatusCode".into(),
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    pub code: StatusCode,
    pub message: String,
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

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self {
            code: StatusCode::IOError,
            message: err.to_string(),
        }
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(err: PoisonError<T>) -> Self {
        Self {
            code: StatusCode::PoisonError,
            message: err.to_string(),
        }
    }
}

impl std::error::Error for Error {}
