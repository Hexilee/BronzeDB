use crate::ext::{ReadKVExt, WriteKVExt};
use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::u8::MAX;
use util::status::{Error, Result, StatusCode};

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

pub enum Request<T: Deref<Target = [u8]>> {
    Set {
        key: T,
        value: T,
    },
    Get {
        key: T,
    },
    Delete {
        key: T,
    },
    Scan {
        lower_bound: Option<T>,
        upper_bound: Option<T>,
    },
}

impl<T: Deref<Target = [u8]>> Request<T> {
    pub fn write_to(self, mut writer: impl Write) -> io::Result<usize> {
        let mut counter = 1usize; // for Action
        match self {
            Request::Set { key, value } => {
                writer.write_u8(Action::Set as u8)?;
                counter += writer.write_key(&key)?;
                counter += writer.write_value(&value)?;
            }

            Request::Get { key } => {
                writer.write_u8(Action::Get as u8)?;
                counter += writer.write_key(&key)?;
            }

            Request::Delete { key } => {
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
        }
        Ok(counter)
    }
}

impl Request<Vec<u8>> {
    pub fn read_from(mut reader: impl Read) -> Result<Self> {
        let action = reader.read_u8()?.into();
        match action {
            Action::Set => Ok(Request::<Vec<u8>>::Set {
                key: reader.read_key()?,
                value: reader.read_value()?,
            }),
            Action::Get => Ok(Request::<Vec<u8>>::Get {
                key: reader.read_key()?,
            }),
            Action::Delete => Ok(Request::<Vec<u8>>::Delete {
                key: reader.read_key()?,
            }),
            Action::Scan => {
                let mut lower_bound = reader.read_key()?;
                let mut upper_bound = reader.read_key()?;
                Ok(Request::<Vec<u8>>::Scan {
                    lower_bound: if lower_bound.as_slice() == MIN_KEY {
                        None
                    } else {
                        Some(lower_bound)
                    },
                    upper_bound: if upper_bound.as_slice() == MAX_KEY {
                        None
                    } else {
                        Some(upper_bound)
                    },
                })
            }
            Action::Unknown => Err(Error::new(StatusCode::UnknownAction, "action is unknown")),
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
        let delete_request = Request::Delete { key: &b"name"[..] };
        assert_eq!(7usize, delete_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::<Vec<u8>>::Delete{ref key}));
        if let Request::<Vec<u8>>::Delete { ref key } = new_request {
            assert_eq!(&b"name"[..], key.as_slice());
        }
    }

    #[test]
    fn request_get_test() {
        let mut buf = Vec::new();
        let get_request = Request::Get { key: &b"name"[..] };
        assert_eq!(7usize, get_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::<Vec<u8>>::Get{ref key}));
        if let Request::<Vec<u8>>::Get { ref key } = new_request {
            assert_eq!(&b"name"[..], key.as_slice());
        }
    }

    #[test]
    fn request_scan_test() {
        let mut buf = Vec::new();
        let scan_request = Request::Scan {
            lower_bound: None,
            upper_bound: Some(&b"name"[..]),
        };
        assert_eq!(265usize, scan_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::<Vec<u8>>::Scan{ref lower_bound, ref upper_bound}));
        if let Request::<Vec<u8>>::Scan {
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
        let set_request = Request::Set {
            key: &b"last_name"[..],
            value: &b"Lee"[..],
        };
        assert_eq!(17usize, set_request.write_to(&mut buf).unwrap());
        let new_request = Request::read_from(&mut Cursor::new(buf)).unwrap();
        assert!(matches!(&new_request, Request::<Vec<u8>>::Set{ref key, ref value}));
        if let Request::<Vec<u8>>::Set { ref key, ref value } = new_request {
            assert_eq!(&b"last_name"[..], key.as_slice());
            assert_eq!(&b"Lee"[..], value.as_slice());
        }
    }
}
