use util::types::{Entry, Key, Value};

pub trait Engine {
    type Error: std::error::Error;
    fn set(&mut self, key: Key, value: Value) -> Result<(), Self::Error>;
    fn get(&self, key: Key) -> Result<Option<Value>, Self::Error>; // TODO: use Result<Option<Value>>
    fn delete(&mut self, key: Key) -> Result<(), Self::Error>;
    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<Box<dyn Scanner + '_>, Self::Error>;
}

pub trait Scanner {
    type Error: std::error::Error;
    fn size(&self) -> usize;
    fn iter(&self) -> Box<dyn Iterator<Item = Result<Entry, Self::Error>> + '_>;
}
