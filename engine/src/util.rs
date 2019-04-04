pub type Key = [u8; 256];
pub type Value = Vec<u8>;

pub struct Entity {
    key: Key,
    value: Value,
}
