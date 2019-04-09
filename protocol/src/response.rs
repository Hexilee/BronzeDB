use super::request::Action::{self, *};
use crate::util::{ReadKVExt, WriteKVExt};
use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::default::Default;
use std::io::{self, Read, Write};
use util::status::StatusCode::{self, UnknownAction, *};
use util::status::{Error, Result};
use util::types::{Entry, Key, Value};

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

    pub fn read_from<'a>(mut reader: impl Read + 'a, request_action: Action) -> Result<Self> {
        match reader.read_u8()?.into() {
            OK => match request_action {
                Get => Ok(Response::SingleValue {
                    status: OK,
                    value: reader.read_value()?,
                }),
                Delete => Ok(Response::Status(OK)),
                Set => Ok(Response::Status(OK)),
                Scan => {
                    let size = reader.read_u64::<BigEndian>()?;
                    unimplemented!()
                }
                Unknown => Err(Error::new(
                    UnknownAction,
                    format!("unknown action: {:?}", request_action),
                )),
            },
            code => Err(Error::new(code, "not ok")),
        }
    }
}

struct ReaderIter<'a> {
    size: usize,
    reader: Box<dyn Read + 'a>,
    entry: (Key, Value),
}

impl<'a> ReaderIter<'a> {
    fn new(size: usize, reader: Box<dyn Read + 'a>) -> Self {
        Self {
            size,
            reader,
            entry: Default::default(),
        }
    }
}

impl Iterator for ReaderIter<'_> {
    type Item = Entry<'_>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            return None;
        }
        // TODO: using Result<Entry<'a>>
        self.entry = (
            self.reader.read_key().unwrap().into(),
            self.reader.read_value().unwrap(),
        );
        self.size -= 1;
        Some((&self.entry.0, &self.entry.1))
    }
}
