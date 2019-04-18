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
    fn set(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.inner.set(key, value)?;
        Ok(())
    }

    fn get(&self, key: Key) -> Result<Vec<u8>> {
        Ok(self.inner.get(key)?.to_owned())
    }

    fn delete(&mut self, key: Key) -> Result<()> {
        unimplemented!()
    }

    fn scan(&self, lower_bound: Option<Key>, upper_bound: Option<Key>) -> Result<Box<dyn Scanner>> {
        unimplemented!()
    }
}
