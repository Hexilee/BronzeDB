#[macro_use(matches)]
extern crate matches;

use crate::util::{Entry, RawKey, Value};

pub trait Engine {
    fn set(key: &RawKey, value: Value) -> status::Result<()>;
    fn get(key: &RawKey, value: &mut Value) -> status::Result<()>;
    fn delete(key: &RawKey) -> status::Result<()>;
    fn scan(
        lower_bound: Option<&RawKey>,
        upper_bound: Option<&RawKey>,
        visitor: impl Fn(&Entry) -> status::Result<()>,
    ) -> status::Result<usize>;
}

pub mod status;
pub mod util;
