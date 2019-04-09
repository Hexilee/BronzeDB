#[macro_use(matches)]
extern crate matches;

use crate::util::{Entry, Key, Value};

pub trait Engine {
    fn set(&mut self, key: Key, value: Value) -> status::Result<()>;
    fn get(&self, key: Key) -> status::Result<Value>;
    fn delete(&mut self, key: Key) -> status::Result<()>;
    fn scan(
        &self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> status::Result<Box<dyn Scan + '_>>;
}

pub trait Scan {
    fn size(&self) -> usize;
    fn iter(&self) -> Box<dyn Iterator<Item = Entry<'_>>>;
}

pub mod status;
pub mod util;
