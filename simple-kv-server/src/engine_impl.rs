use engine::status;
use engine::util::{Entry, Key, Value};
use engine::Engine;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct EngineImpl {
    inner: Arc<RwLock<HashMap<Key, Value>>>,
}

impl EngineImpl {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Engine for EngineImpl {
    fn set(&mut self, key: Key, value: Vec<u8>) -> status::Result<()> {
        self.inner.write()?.insert(key, value);
        Ok(())
    }

    fn get(&self, key: Key) -> status::Result<Value> {
        Ok(self
            .inner
            .read()?
            .get(&key)
            .ok_or(status::Error::new(
                status::StatusCode::NotFound,
                format!("key {:?} not found", &key),
            ))?
            .clone())
    }

    fn delete(&mut self, key: Key) -> status::Result<()> {
        self.inner.write()?.remove(&key).ok_or(status::Error::new(
            status::StatusCode::NotFound,
            format!("key {:?} not found", &key),
        ))?;
        Ok(())
    }

    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
        visitor: impl Fn(Entry) -> status::Result<()>,
    ) -> status::Result<usize> {
        let mut counter = 0usize;
        let inner = self.inner.read()?;
        let mut entries: Box<dyn Iterator<Item = Entry>> = Box::new(inner.iter());
        if let Some(lower_key) = lower_bound {
            entries = Box::new(entries.filter(move |(key, _)| *key >= &lower_key))
        }
        if let Some(upper_key) = upper_bound {
            entries = Box::new(entries.filter(move |(key, _)| *key <= &upper_key))
        }
        for (key, value) in entries.into_iter() {
            visitor((key, value))?;
            counter += 1;
        }
        Ok(counter)
    }
}
