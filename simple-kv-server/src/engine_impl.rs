use engine::util::{Entry, Key, Value};
use engine::Engine;
use engine::{status, Scan};
use std::collections::HashMap;
use std::sync::{Arc, RwLock, RwLockReadGuard};

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
    ) -> status::Result<Box<dyn Scan + '_>> {
        let guard: RwLockReadGuard<'_, HashMap<Key, Value>> = self.inner.read()?;
        let mut scan = GuardScan {
            guard,
            lower_bound,
            upper_bound,
            size: 0,
        };

        let mut counter = 0;
        for _ in scan.filter_entries(Box::new(scan.guard.iter())) {
            counter += 1;
        }
        scan.size = counter;
        Ok(Box::new(scan))
    }
}

struct GuardScan<'a> {
    guard: RwLockReadGuard<'a, HashMap<Key, Value>>,
    size: usize,
    lower_bound: Option<Key>,
    upper_bound: Option<Key>,
}

impl GuardScan<'_> {
    fn filter_entries<'a>(
        &'a self,
        mut entries: Box<dyn Iterator<Item = Entry<'a>> + 'a>,
    ) -> Box<dyn Iterator<Item = Entry<'a>> + 'a> {
        if let Some(lower_key) = self.lower_bound.as_ref() {
            entries = Box::new(entries.filter(move |(key, _)| *key >= lower_key))
        }
        if let Some(upper_key) = self.upper_bound.as_ref() {
            entries = Box::new(entries.filter(move |(key, _)| *key <= upper_key))
        }
        entries
    }
}

impl Scan for GuardScan<'_> {
    fn size(&self) -> usize {
        self.size
    }

    fn iter(&self) -> Box<Iterator<Item = Entry<'_>>> {
        unimplemented!()
    }
}
