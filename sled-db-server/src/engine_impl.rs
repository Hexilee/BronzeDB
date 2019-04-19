use engine::{Engine, Scanner};
use sled::Db;
use std::path;
use util::status::{Error, StatusCode};
use util::types::Key;

#[derive(Debug)]
pub struct EngineError {
    inner: sled::Error,
}

impl EngineError {
    pub fn new(err: sled::Error) -> Self {
        Self { inner: err }
    }
}

impl From<sled::Error> for EngineError {
    fn from(err: sled::Error) -> Self {
        Self::new(err)
    }
}

impl From<EngineError> for Error {
    fn from(err: EngineError) -> Self {
        Error::new(StatusCode::EngineError, err.inner.to_string())
    }
}

#[derive(Clone)]
pub struct EngineImpl {
    inner: Db,
}

impl EngineImpl {
    pub fn new(path: impl AsRef<path::Path>) -> Self {
        Self {
            inner: Db::start_default(path.as_ref())
                .expect(&format!("cannot open db {:?}", path.as_ref())),
        }
    }
}

impl Engine for EngineImpl {
    type Error = EngineError;

    fn set(&mut self, key: Key, value: Vec<u8>) -> Result<(), Self::Error> {
        self.inner.set(key, value)?;
        Ok(())
    }

    fn get(&self, key: Key) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self.inner.get(key)?.map(|data| data.to_vec()))
    }

    fn delete(&mut self, key: Key) -> Result<(), Self::Error> {
        self.inner.del(key)?;
        Ok(())
    }

    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<Box<Scanner>, Self::Error> {
        self.inner.scan()
    }
}

pub struct SledScanner {
    //    iter
}
