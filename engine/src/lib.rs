#[macro_use(matches)]
extern crate matches;

use crate::util::{Entry, Key, Value};

pub trait Engine {
    fn set(key: Key, value: &Value) -> status::Result<()>;
    fn get(key: Key, value: &mut Value) -> status::Result<()>;
    fn delete(key: Key) -> status::Result<()>;
    fn scan(
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
        visitor: impl Fn(&Entry) -> status::Result<()>,
    ) -> status::Result<usize>;
}

pub mod status;
pub mod util;
