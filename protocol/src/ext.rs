use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use util::types::{Key, Value};

pub trait WriteKVExt: Write {
    fn write_key(&mut self, key: &[u8]) -> io::Result<usize>;
    fn write_value(&mut self, key: &[u8]) -> io::Result<usize>;
}

pub trait ReadKVExt: Read {
    fn read_key(&mut self) -> io::Result<Vec<u8>>;
    fn read_value(&mut self) -> io::Result<Value>;
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

impl<T: Read + ?Sized> ReadKVExt for T {
    fn read_key(&mut self) -> io::Result<Vec<u8>> {
        let key_len = self.read_u16::<BigEndian>()? as usize;
        debug_assert!(key_len <= MAX_KEY_LEN);
        let mut key = Vec::with_capacity(key_len);
        unsafe { key.set_len(key_len) };
        self.read_exact(key.as_mut_slice())?;
        Ok(key)
    }

    fn read_value(&mut self) -> io::Result<Value> {
        let value_len = self.read_u16::<BigEndian>()? as usize;
        debug_assert!(value_len <= MAX_VALUE_LEN);
        let mut value = Vec::with_capacity(value_len);
        unsafe { value.set_len(value_len) };
        self.read_exact(value.as_mut_slice())?;
        Ok(value)
    }
}
