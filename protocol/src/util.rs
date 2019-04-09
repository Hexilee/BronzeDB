use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Write};

pub trait WriteKVExt: Write {
    fn write_key(&mut self, key: &[u8]) -> io::Result<usize>;
    fn write_value(&mut self, key: &[u8]) -> io::Result<usize>;
}

impl<T: Write + ?Sized> WriteKVExt for T {
    fn write_key(&mut self, key: &[u8]) -> io::Result<usize> {
        debug_assert!(key.len() <= MAX_KEY_LEN);
        self.write_u16::<BigEndian>(key.len() as u16)?;
        self.write_all(key)?;
        Ok(2 + key.len())
    }

    fn write_value(&mut self, value: &[u8]) -> io::Result<usize> {
        debug_assert!(value.len() <= MAX_VALUE_LEN);
        self.write_u16::<BigEndian>(value.len() as u16)?;
        self.write_all(value)?;
        Ok(2 + value.len())
    }
}
