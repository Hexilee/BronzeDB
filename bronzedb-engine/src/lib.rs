use util::status::Error;
use util::types::{Entry, Key, Value};

pub trait Engine {
    type Error: Into<Error>;
    fn set(&mut self, key: Key, value: Value) -> Result<(), Self::Error>;
    fn get(&self, key: Key) -> Result<Option<Value>, Self::Error>;
    fn delete(&mut self, key: Key) -> Result<(), Self::Error>;
    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<Box<dyn Scanner + '_>, Self::Error>;
}

pub trait Scanner {
    fn iter(&mut self) -> Box<dyn Iterator<Item = Result<Entry, Error>> + '_>;
}
