use util::status::{Error, Result};
use util::types::{Entry, Key, Value};

pub trait Engine {
    fn set(&mut self, key: Key, value: Value) -> Result<()>;
    fn get(&self, key: Key) -> Result<Option<Value>>;
    fn delete(&mut self, key: Key) -> Result<()>;
    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<Box<dyn Scanner + '_>>;
}

pub trait Scanner {
    fn size(&self) -> usize;
    fn iter(&self) -> Box<dyn Iterator<Item = Result<Entry>> + '_>;
}
