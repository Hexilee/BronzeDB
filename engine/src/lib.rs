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
        visitor: impl Fn(Entry) -> status::Result<()>,
    ) -> status::Result<usize>;
}

pub mod status;
pub mod util;
