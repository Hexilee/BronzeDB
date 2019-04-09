use crate::{MAX_KEY, MAX_KEY_LEN, MAX_VALUE_LEN, MIN_KEY};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use engine::err;
use std::io::{self, Read, Write};
use std::ops::Deref;

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Set = 0,
    Get = 1,
    Delete = 2,
    Scan = 3,
    Unknown = 4,
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
                debug_assert!(key.len() <= MAX_KEY_LEN);
                debug_assert!(value.len() <= MAX_VALUE_LEN);
                writer.write_u16::<BigEndian>(key.len() as u16)?;
                writer.write_u16::<BigEndian>(value.len() as u16)?;
                writer.write_all(&key)?;
                writer.write_all(&value)?;
                counter += 4 + key.len() + value.len();
            }

            Request::Get { key } => {
                writer.write_u8(Action::Get as u8)?;
                debug_assert!(key.len() <= MAX_KEY_LEN);
                writer.write_u16::<BigEndian>(key.len() as u16)?;
                writer.write_all(&key)?;
                counter += 2 + key.len()
            }

            Request::Delete { key } => {
                writer.write_u8(Action::Delete as u8)?;
                debug_assert!(key.len() <= MAX_KEY_LEN);
                writer.write_u16::<BigEndian>(key.len() as u16)?;
                writer.write_all(&key)?;
                counter += 2 + key.len()
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
                debug_assert!(lower_key.len() <= MAX_KEY_LEN);
                debug_assert!(upper_key.len() <= MAX_KEY_LEN);
                writer.write_u16::<BigEndian>(lower_key.len() as u16)?;
                writer.write_u16::<BigEndian>(upper_key.len() as u16)?;
                writer.write_all(&lower_key)?;
                writer.write_all(&upper_key)?;
                counter += 4 + lower_key.len() + upper_key.len();
            }
        }
        Ok(counter)
    }
}

impl Request<Vec<u8>> {
    pub fn read_from(mut reader: impl Read) -> err::Result<Self> {
        let action = reader.read_u8()?.into();
        match action {
            Action::Set => {
                let key_len = reader.read_u16::<BigEndian>()?;
                let value_len = reader.read_u16::<BigEndian>()?;
                let mut key = Vec::with_capacity(key_len as usize);
                unsafe { key.set_len(key_len as usize) };
                reader.read_exact(key.as_mut_slice())?;
                let mut value = Vec::with_capacity(value_len as usize);
                unsafe { value.set_len(value_len as usize) };
                reader.read_exact(value.as_mut_slice())?;
                Ok(Request::<Vec<u8>>::Set { key, value })
            }

            Action::Get => {
                let key_len = reader.read_u16::<BigEndian>()?;
                let mut key = Vec::with_capacity(key_len as usize);
                unsafe { key.set_len(key_len as usize) };
                reader.read_exact(key.as_mut_slice())?;
                Ok(Request::<Vec<u8>>::Get { key })
            }

            Action::Delete => {
                let key_len = reader.read_u16::<BigEndian>()?;
                let mut key = Vec::with_capacity(key_len as usize);
                unsafe { key.set_len(key_len as usize) };
                reader.read_exact(key.as_mut_slice())?;
                Ok(Request::<Vec<u8>>::Delete { key })
            }

            Action::Scan => {
                let lower_bound_len = reader.read_u16::<BigEndian>()?;
                let upper_bound_len = reader.read_u16::<BigEndian>()?;
                let mut lower_bound = Vec::with_capacity(lower_bound_len as usize);
                unsafe { lower_bound.set_len(lower_bound_len as usize) };
                reader.read_exact(lower_bound.as_mut_slice())?;
                let mut upper_bound = Vec::with_capacity(upper_bound_len as usize);
                unsafe { upper_bound.set_len(upper_bound_len as usize) };
                reader.read_exact(upper_bound.as_mut_slice())?;
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
            Action::Unknown => Err(err::Error::new(
                err::StatusCode::UnknownAction,
                "action is unknown",
            )),
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
