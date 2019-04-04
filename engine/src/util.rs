use num_bigint::BigUint;

pub type Key = BigUint;
pub type Value = Vec<u8>;

pub type Entry = (Key, Value);
