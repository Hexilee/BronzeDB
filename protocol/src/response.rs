use crate::util::WriteKVExt;
use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Write};
use util::status::StatusCode;
use util::types::{Entry, Value};

pub enum Response<'a> {
    Status(StatusCode),
    SingleValue {
        status: StatusCode,
        value: Value,
    },
    MultiKV {
        status: StatusCode,
        size: usize,
        iter: Box<dyn Iterator<Item = Entry<'a>> + 'a>,
    },
}

impl Response<'_> {
    pub fn write_to(self, mut writer: impl Write) -> io::Result<usize> {
        let mut counter = 1usize; // for StatusCode
        match self {
            Response::Status(status) => writer.write_u8(status as u8)?,
            Response::SingleValue { status, value } => {
                writer.write_u8(status as u8)?;
                counter += writer.write_value(&value)?;
            }
            Response::MultiKV { status, size, iter } => {
                writer.write_u8(status as u8)?;
                writer.write_u64::<BigEndian>(size as u64)?;
                counter += 8;
                for (key, value) in iter {
                    counter += writer.write_key(key)?;
                    counter += writer.write_value(value)?;
                }
            }
        }
        Ok(counter)
    }
}
