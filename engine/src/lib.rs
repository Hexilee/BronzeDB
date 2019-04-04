use crate::util::{Entry, Key, Value};

pub trait Engine {
    fn set(entities: impl Iterator<Item = Entry>) -> status::Result<usize>;
    fn get(key: &Key) -> status::Result<Value>;
    fn delete(keys: impl Iterator<Item = Key>) -> status::Result<usize>;
    fn scan(
        lower_bound: Option<&Key>,
        upper_bound: Option<&Key>,
        visitor: impl Fn(&Entry) -> status::Result<()>,
    ) -> status::Result<usize>;
}

pub mod status;
pub mod util;
