use super::request::Action::{self, *};
use crate::util::{ReadKVExt, WriteKVExt};
use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use util::status::StatusCode::{self, *};
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
        iter: Box<dyn Iterator<Item = Result<Entry>> + 'a>,
    },
}

impl<'a> Response<'a> {
    pub fn write_to(self, mut writer: impl Write) -> Result<usize> {
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
                for result in iter {
                    match result {
                        Ok((key, value)) => {
                            writer.write_u8(OK as u8)?;
                            counter += 1 + writer.write_key(&key)? + writer.write_value(&value)?;
                        }
                        Err(err) => {
                            writer.write_u8(err.code as u8)?;
                            Err(err)?
                        }
                    }
                }
            }
        }
        Ok(counter)
    }

    pub fn read_from(mut reader: impl Read + 'a, request_action: Action) -> Result<Self> {
        match reader.read_u8()?.into() {
            OK => match request_action {
                Get => Ok(Response::SingleValue {
                    status: OK,
                    value: reader.read_value()?,
                }),
                Delete => Ok(Response::Status(OK)),
                Set => Ok(Response::Status(OK)),
                Scan => {
                    let size = reader.read_u64::<BigEndian>()? as usize;
                    Ok(Response::MultiKV {
                        status: OK,
                        size,
                        iter: Box::new(ReaderIter::new(size, Box::new(reader))),
                    })
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
}

impl<'a> ReaderIter<'a> {
    fn new(size: usize, reader: Box<dyn Read + 'a>) -> Self {
        Self { size, reader }
    }
}

impl Iterator for ReaderIter<'_> {
    type Item = Result<Entry>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            return None;
        }
        self.size -= 1;
        Some(pair_result(
            self.reader.read_key(),
            self.reader.read_value(),
        ))
    }
}

fn pair_result(key: io::Result<Vec<u8>>, value: io::Result<Vec<u8>>) -> Result<Entry> {
    Ok((key?.into(), value?))
}
