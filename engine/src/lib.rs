use crate::util::{Key, Value, KV};

pub trait Engine {
    fn set(entities: impl Iterator<Item = KV>) -> status::Result<usize>;
    fn get(key: &Key) -> status::Result<Value>;
    fn delete(keys: impl Iterator<Item = Key>) -> status::Result<usize>;
    fn scan(
        lower_bound: Option<&Key>,
        upper_bound: Option<&Key>,
        visitor: impl Fn(&KV) -> status::Result<()>,
    ) -> status::Result<usize>;
}

pub mod status;
pub mod util;
