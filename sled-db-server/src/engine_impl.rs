use engine::{Engine, Scanner};
use sled::Db;
use util::status::Result;
use util::types::Key;

#[derive(Clone)]
pub struct EngineImpl {
    inner: Db,
}

impl EngineImpl {
    pub fn new(path: impl AsRef<std::path::Path>) -> Self {
        Self {
            inner: Db::start_default(path.as_ref())
                .expect(&format!("cannot open db {:?}", path.as_ref())),
        }
    }
}

impl Engine for EngineImpl {
    type Error = ();

    fn set(&mut self, key: Key, value: Vec<u8>) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn get(&self, key: Key) -> Result<Option<Vec<u8>>, Self::Error> {
        unimplemented!()
    }

    fn delete(&mut self, key: Key) -> Result<(), Self::Error> {
        unimplemented!()
    }

    fn scan(&self, lower_bound: Option<Key>, upper_bound: Option<Key>) -> Result<Box<Scanner>, Self::Error> {
        unimplemented!()
    }
}
