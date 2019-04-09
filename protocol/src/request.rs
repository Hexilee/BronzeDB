use crate::ext::{ReadKVExt, WriteKVExt};
use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::u8::MAX;
use util::status::{Error, Result, StatusCode};
use util::types::{Entry, Key, Value};

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Set = 0,
    Get = 1,
    Delete = 2,
    Scan = 3,
    Unknown = MAX as isize,
}

impl From<u8> for Action {
    fn from(value: u8) -> Self {
        match value {
            0 => Action::Set,
            1 => Action::Get,
            2 => Action::Delete,
            3 => Action::Scan,
            _ => Action::Unknown,
        }
    }
}

pub enum Request {
    Set(Key, Value),
    Get(Key),
    Delete(Key),
    Scan {
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    },
    Unknown,
}

impl Request {
    pub fn write_to(self, mut writer: impl Write) -> io::Result<usize> {
        let mut counter = 1usize; // for Action
        match self {
            Request::Set(key, value) => {
                writer.write_u8(Action::Set as u8)?;
                counter += writer.write_key(&key)?;
                counter += writer.write_value(&value)?;
            }

            Request::Get(key) => {
                writer.write_u8(Action::Get as u8)?;
                counter += writer.write_key(&key)?;
            }

            Request::Delete(key) => {
                writer.write_u8(Action::Delete as u8)?;
                counter += writer.write_key(&key)?;
            }

            Request::Scan {
                lower_bound,
                upper_bound,
            } => {
                writer.write_u8(Action::Scan as u8)?;
                let lower_key = match lower_bound.as_ref() {
                    Some(key) => key.deref(),
                    None => MIN_KEY,
                };
                let upper_key = match upper_bound.as_ref() {
                    Some(key) => key.deref(),
                    None => MIN_KEY,
                };
                counter += writer.write_key(&lower_key)?;
                counter += writer.write_key(&upper_key)?;
            }

            Request::Unknown => panic!("cannot send Request::Unknown"),
        }
        Ok(counter)
    }
}

impl Request {
    pub fn read_from(mut reader: impl Read) -> Result<Self> {
        let action = reader.read_u8()?.into();
        match action {
            Action::Set => Ok(Request::Set(
                reader.read_key()?.into(),
                reader.read_value()?,
            )),
            Action::Get => Ok(Request::Get(reader.read_key()?.into())),

            Action::Delete => Ok(Request::Delete(reader.read_key()?.into())),
            Action::Scan => {
                let lower_bound = reader.read_key()?;
                let upper_bound = reader.read_key()?;
                Ok(Request::Scan {
                    lower_bound: if lower_bound.as_slice() == MIN_KEY {
                        None
                    } else {
                        Some(lower_bound.into())
                    },
                    upper_bound: if upper_bound.as_slice() == MAX_KEY {
                        None
                    } else {
                        Some(upper_bound.into())
                    },
                })
            }
            Action::Unknown => Ok(Request::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Request;
    use std::io::{self, Cursor, Read};

    #[test]
    fn change_vec_as_mut_slice() {
        let mut data = Vec::with_capacity(3);
        unsafe { data.set_len(3) };
        io::repeat(1).read_exact(data.as_mut_slice()).unwrap();
        assert_eq!(3, data.len());
        assert_eq!(&[1u8, 1, 1], &data[..]);
    }

    #[test]
    fn request_delete_test() {
        let mut buf = Vec::new();
        let delete_request = Request::Delete(b"name"[..].to_vec().into());
        assert_eq!(7usize, delete_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::Delete(ref key)));
        if let Request::Delete(ref key) = new_request {
            assert_eq!(&b"name"[..], key.as_slice());
        }
    }

    #[test]
    fn request_get_test() {
        let mut buf = Vec::new();
        let get_request = Request::Get(b"name"[..].to_vec().into());
        assert_eq!(7usize, get_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::Get(ref key)));
        if let Request::Get(ref key) = new_request {
            assert_eq!(&b"name"[..], key.as_slice());
        }
    }

    #[test]
    fn request_scan_test() {
        let mut buf = Vec::new();
        let scan_request = Request::Scan {
            lower_bound: None,
            upper_bound: Some(b"name"[..].to_vec().into()),
        };
        assert_eq!(265usize, scan_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::Scan{ref lower_bound, ref upper_bound}));
        if let Request::Scan {
            lower_bound,
            upper_bound,
        } = new_request
        {
            assert!(matches!(lower_bound, None));
            assert!(matches!(upper_bound, Some(key)));
        }
    }

    #[test]
    fn request_set_test() {
        let mut buf = Vec::new();
        let set_request =
            Request::Set(b"last_name"[..].to_vec().into(), b"Lee"[..].to_vec().into());
        assert_eq!(17usize, set_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::Set(ref key, ref value)));
        if let Request::Set(ref key, ref value) = new_request {
            assert_eq!(&b"last_name"[..], key.as_slice());
            assert_eq!(&b"Lee"[..], value.as_slice());
        }
    }
}
