use engine::status;
use engine::util::{Entry, Key, Value};
use engine::Engine;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
    fn set(key: &Key, value: Vec<u8>) -> status::Result<()> {
        unimplemented!()
    }

    fn get(key: &Key, value: &mut Vec<u8>) -> status::Result<()> {
        unimplemented!()
    }

    fn delete(key: &Key) -> status::Result<()> {
        unimplemented!()
    }

    fn scan(
        lower_bound: Option<&Key>,
        upper_bound: Option<&Key>,
        visitor: impl Fn(&Entry) -> status::Result<()>,
    ) -> status::Result<usize> {
        unimplemented!()
    }
}
