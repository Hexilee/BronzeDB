#[macro_use(matches)]
extern crate matches;

use crate::util::{Entry, Key, Value};

pub trait Engine {
    fn set(&mut self, key: Key, value: Value) -> err::Result<()>;
    fn get(&self, key: Key) -> err::Result<Value>;
    fn delete(&mut self, key: Key) -> err::Result<()>;
    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> err::Result<Box<dyn Scan + '_>>;
}

pub trait Scan {
    fn size(&self) -> usize;
    fn iter(&self) -> Box<dyn Iterator<Item = Entry<'_>> + '_>;
}
