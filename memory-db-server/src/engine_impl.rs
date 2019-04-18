use engine::{Engine, Scanner};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::PoisonError;
use std::sync::{Arc, RwLock, RwLockReadGuard};
use util::status::{Error, StatusCode};
use util::types::{Entry, EntryRef, Key, Value};

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

#[derive(Debug)]
pub enum EngineError {
    PoisonError(String),
}

impl<T> From<PoisonError<T>> for EngineError {
    fn from(poison_err: PoisonError<T>) -> Self {
        EngineError::PoisonError(poison_err.to_string())
    }
}

impl From<EngineError> for Error {
    fn from(err: EngineError) -> Self {
        Error::new(StatusCode::EngineError, format!("engine error: {}", err))
    }
}

impl Display for EngineError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            EngineError::PoisonError(ref err) => f.write_str(&format!("PoisonError: {}", err)),
        }
    }
}

impl std::error::Error for EngineError {}

impl Engine for EngineImpl {
    type Error = EngineError;
    fn set(&mut self, key: Key, value: Vec<u8>) -> Result<(), Self::Error> {
        self.inner.write()?.insert(key, value);
        Ok(())
    }

    fn get(&self, key: Key) -> Result<Option<Value>, Self::Error> {
        Ok(self.inner.read()?.get(&key).map(|value| value.clone()))
    }

    fn delete(&mut self, key: Key) -> Result<(), Self::Error> {
        self.inner.write()?.remove(&key);
        Ok(())
    }

    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<Box<dyn Scanner + '_>, Self::Error> {
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
    fn entries(&self) -> Box<dyn Iterator<Item = Result<Entry, Error>> + '_> {
        let mut entries: Box<dyn Iterator<Item = EntryRef>> = Box::new(self.guard.iter());
        if let Some(lower_key) = self.lower_bound.as_ref() {
            entries = Box::new(entries.filter(move |(key, _)| *key >= lower_key))
        }
        if let Some(upper_key) = self.upper_bound.as_ref() {
            entries = Box::new(entries.filter(move |(key, _)| *key <= upper_key))
        }
        Box::new(entries.map(|(key, value)| Ok((key.clone(), value.clone()))))
    }
}

impl Scanner for GuardScanner<'_> {
    fn size(&self) -> usize {
        self.size
    }

    fn iter(&self) -> Box<dyn Iterator<Item = Result<Entry, Error>> + '_> {
        self.entries()
    }
}
