use std::fmt::{Display, Formatter};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub struct Error {
    code: StatusCode,
    message: String,
}

#[derive(Debug, Copy, Clone)]
pub enum StatusCode {}

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

impl std::error::Error for Error {}
