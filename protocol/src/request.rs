use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Write};

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Set = 0,
    Get = 1,
    Delete = 2,
    Scan = 3,
}

pub enum Request<'a> {
    Set(Vec<(&'a [u8], &'a [u8])>),
    Get(Vec<&'a [u8]>),
    Delete(Vec<&'a [u8]>),
    Scan {
        lower_bound: Option<&'a [u8]>,
        upper_bound: Option<&'a [u8]>,
    },
}

impl<'a> Request<'a> {
    pub fn write_to(self, mut writer: impl Write) -> io::Result<usize> {
        let mut counter = 1usize; // for Action
        match self {
            Request::Set(entries) => {
                writer.write_u8(Action::Set as u8)?;
                writer.write_u32::<BigEndian>(entries.len() as u32)?;
                counter += 4 + 4 * entries.len();
                for (key, value) in entries {
                    debug_assert!(key.len() <= MAX_KEY_LEN);
                    debug_assert!(value.len() <= MAX_VALUE_LEN);
                    writer.write_u16::<BigEndian>(key.len() as u16)?;
                    writer.write_u16::<BigEndian>(value.len() as u16)?;
                    writer.write_all(key)?;
                    writer.write_all(value)?;
                    counter += key.len() + value.len();
                }
            }

            Request::Get(keys) => {
                writer.write_u8(Action::Get as u8)?;
                writer.write_u32::<BigEndian>(keys.len() as u32)?;
                counter += 4 + 2 * keys.len();
                for key in keys {
                    debug_assert!(key.len() <= MAX_KEY_LEN);
                    writer.write_u16::<BigEndian>(key.len() as u16)?;
                    writer.write_all(key)?;
                    counter += key.len()
                }
            }

            Request::Delete(keys) => {
                writer.write_u8(Action::Delete as u8)?;
                writer.write_u32::<BigEndian>(keys.len() as u32)?;
                counter += 4 + 2 * keys.len();
                for key in keys {
                    debug_assert!(key.len() <= MAX_KEY_LEN);
                    writer.write_u16::<BigEndian>(key.len() as u16)?;
                    writer.write_all(key)?;
                    counter += key.len()
                }
            }

            Request::Scan {
                lower_bound,
                upper_bound,
            } => {
                writer.write_u8(Action::Scan as u8)?;
                let lower_key = lower_bound.unwrap_or(MIN_KEY);
                let upper_key = upper_bound.unwrap_or(MAX_KEY);
                debug_assert!(lower_key.len() <= MAX_KEY_LEN);
                debug_assert!(upper_key.len() <= MAX_KEY_LEN);
                writer.write_u16::<BigEndian>(lower_key.len() as u16)?;
                writer.write_u16::<BigEndian>(upper_key.len() as u16)?;
                writer.write_all(lower_key)?;
                writer.write_all(upper_key)?;
                counter += 4 + lower_key.len() + upper_key.len();
            }
        }
        Ok(counter)
    }
}
