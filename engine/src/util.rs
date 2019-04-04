use num_bigint::BigUint;

pub type Key = BigUint;
pub type Value = Vec<u8>;

pub type KV = (Key, Value);
