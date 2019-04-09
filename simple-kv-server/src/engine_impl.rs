use engine::{Engine, Scanner};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use util::status::{Error, Result, StatusCode};
use util::types::{Entry, Key, Value};

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
    fn set(&mut self, key: Key, value: Vec<u8>) -> Result<()> {
        self.inner.write()?.insert(key, value);
        Ok(())
    }

    fn get(&self, key: Key) -> Result<Value> {
        Ok(self
            .inner
            .read()?
            .get(&key)
            .ok_or(Error::new(
                StatusCode::NotFound,
                format!("key {:?} not found", &key),
            ))?
            .clone())
    }

    fn delete(&mut self, key: Key) -> Result<()> {
        self.inner.write()?.remove(&key).ok_or(Error::new(
            StatusCode::NotFound,
            format!("key {:?} not found", &key),
        ))?;
        Ok(())
    }

    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<Box<dyn Scanner + '_>> {
        let guard: RwLockReadGuard<'_, HashMap<Key, Value>> = self.inner.read()?;
        let mut scan = GuardScanner {
            guard,
            lower_bound,
            upper_bound,
            size: 0,
        };

        let mut counter = 0;
        for _ in scan.entries() {
            counter += 1;
        }
        scan.size = counter;
        Ok(Box::new(scan))
    }
}

struct GuardScanner<'a> {
    guard: RwLockReadGuard<'a, HashMap<Key, Value>>,
    size: usize,
    lower_bound: Option<Key>,
    upper_bound: Option<Key>,
}

impl GuardScanner<'_> {
    fn entries(&'_ self) -> Box<dyn Iterator<Item = Entry<'_>> + '_> {
        let mut entries: Box<dyn Iterator<Item = Entry>> = Box::new(self.guard.iter());
        if let Some(lower_key) = self.lower_bound.as_ref() {
            entries = Box::new(entries.filter(move |(key, _)| *key >= lower_key))
        }
        if let Some(upper_key) = self.upper_bound.as_ref() {
            entries = Box::new(entries.filter(move |(key, _)| *key <= upper_key))
        }
        entries
    }
}

impl Scanner for GuardScanner<'_> {
    fn size(&self) -> usize {
        self.size
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Entry<'_>> + '_> {
        self.entries()
    }
}
