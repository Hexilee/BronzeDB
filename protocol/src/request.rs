use crate::ext::{ReadKVExt, WriteKVExt};
use crate::{MAX_KEY, MIN_KEY};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{self, Read, Write};
use std::ops::Deref;
use std::u8::MAX;
use util::types::{Key, Value};

#[derive(Debug, Copy, Clone)]
pub enum Action {
    Set = 0,
    Get = 1,
    Delete = 2,
    Scan = 3,
    Ping = 4,
    Unknown = MAX as isize,
}

impl From<u8> for Action {
    fn from(value: u8) -> Self {
        match value {
            0 => Action::Set,
            1 => Action::Get,
            2 => Action::Delete,
            3 => Action::Scan,
            4 => Action::Ping,
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
    Ping,
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
                    None => MAX_KEY,
                };
                counter += writer.write_key(&lower_key)?;
                counter += writer.write_key(&upper_key)?;
            }

            Request::Ping => writer.write_u8(Action::Ping as u8)?,
            Request::Unknown => panic!("cannot send Request::Unknown"),
        }
        Ok(counter)
    }
}

impl Request {
    pub fn read_from(mut reader: impl Read) -> io::Result<Self> {
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
            Action::Ping => Ok(Request::Ping),
            Action::Unknown => Ok(Request::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Request;
    use crate::{MAX_KEY_LEN, MAX_VALUE_LEN};
    use matches::matches;
    use speculate::speculate;
    use std::io::Cursor;
    use util::status::Result;

    pub trait RequestTestExt: Sized {
        fn transfer_move(self) -> Result<(Self, usize)>;
    }

    impl RequestTestExt for Request {
        fn transfer_move(self) -> Result<(Self, usize)> {
            let mut buf = Vec::new();
            let bytes = self.write_to(&mut buf)?;
            Ok((Request::read_from(&mut Cursor::new(buf))?, bytes))
        }
    }

    macro_rules! assert_delete {
        ($data:expr) => {
            let (new_request, bytes) = Request::Delete($data[..].to_vec().into())
                .transfer_move()
                .unwrap();
            assert_eq!($data[..].len() + 3, bytes);
            assert!(matches!(&new_request, Request::Delete(ref _key)));
            if let Request::Delete(ref key) = new_request {
                assert_eq!(&$data[..], key.as_slice());
            }
        };
    }

    speculate! {
        describe "delete" {
            it "normal" {
                assert_delete!(b"name");
            }

            it "zero" {
                assert_delete!([0; 0]);
            }

            it "max length" {
                assert_delete!([0; MAX_KEY_LEN]);
            }

            #[should_panic]
            it "overflow" {
                assert_delete!([0; MAX_KEY_LEN + 1]);
            }
        }
    }

    macro_rules! assert_get {
        ($data:expr) => {
            let (new_request, bytes) = Request::Get($data[..].to_vec().into())
                .transfer_move()
                .unwrap();
            assert_eq!($data[..].len() + 3, bytes);
            assert!(matches!(&new_request, Request::Get(ref _key)));
            if let Request::Get(ref key) = new_request {
                assert_eq!(&$data[..], key.as_slice());
            }
        };
    }

    speculate! {
        describe "get" {
            it "normal" {
                assert_get!(b"name");
            }

            it "zero" {
                assert_get!([0; 0]);
            }

            it "max length" {
                assert_get!([0; MAX_KEY_LEN]);
            }

            #[should_panic]
            it "overflow" {
                assert_get!([0; MAX_KEY_LEN + 1]);
            }
        }
    }

    macro_rules! assert_ping {
        () => {
            let (new_request, bytes) = Request::Ping.transfer_move().unwrap();
            assert_eq!(1, bytes);
            assert!(matches!(new_request, Request::Ping));
        };
    }

    speculate! {
        it "just ping" {
            assert_ping!();
        }
    }

    macro_rules! assert_scan {
        () => {
            let (new_request, bytes) = Request::Scan {
                lower_bound: None,
                upper_bound: None,
            }
            .transfer_move()
            .unwrap();
            assert_eq!(1 + 4 + MAX_KEY_LEN * 2, bytes);
            assert!(matches!(&new_request, Request::Scan{lower_bound: _, upper_bound: _}));
            if let Request::Scan {
                lower_bound,
                upper_bound,
            } = new_request
            {
                assert!(matches!(lower_bound, None));
                assert!(matches!(upper_bound, None));
            }
        };

        ($lower_bound:expr, $upper_bound:expr) => {
            let (new_request, bytes) = Request::Scan {
                lower_bound: Some($lower_bound[..].to_vec().into()),
                upper_bound: Some($upper_bound[..].to_vec().into()),
            }
            .transfer_move()
            .unwrap();
            assert_eq!(1 + 4 + $lower_bound[..].len() + $upper_bound[..].len(), bytes);
            assert!(matches!(&new_request, Request::Scan{lower_bound: _, upper_bound: _}));
            if let Request::Scan {
                lower_bound,
                upper_bound,
            } = new_request
            {
                assert!(matches!(lower_bound, Some(ref _key)));
                assert!(matches!(upper_bound, Some(ref _key)));
                assert_eq!(&$lower_bound[..], lower_bound.unwrap().as_slice());
                assert_eq!(&$upper_bound[..], upper_bound.unwrap().as_slice());
            }
        };

        ($any_bound:expr) => {
            { // lower_bound
                let (new_request, bytes) = Request::Scan {
                    lower_bound: Some($any_bound[..].to_vec().into()),
                    upper_bound: None,
                }
                .transfer_move()
                .unwrap();
                assert_eq!(1 + 4 + $any_bound[..].len() + MAX_KEY_LEN, bytes);
                assert!(matches!(&new_request, Request::Scan{lower_bound: _, upper_bound: _}));
                if let Request::Scan {
                    lower_bound,
                    upper_bound,
                } = new_request
                {
                    assert!(matches!(lower_bound, Some(ref _key)));
                    assert!(matches!(upper_bound, None));
                    assert_eq!(&$any_bound[..], lower_bound.unwrap().as_slice());
                }
            }
            { // upper_bound
                let (new_request, bytes) = Request::Scan {
                    lower_bound: None,
                    upper_bound: Some($any_bound[..].to_vec().into()),
                }
                .transfer_move()
                .unwrap();
                assert_eq!(1 + 4 + MAX_KEY_LEN + $any_bound[..].len(), bytes);
                assert!(matches!(&new_request, Request::Scan{lower_bound: _, upper_bound: _}));
                if let Request::Scan {
                    lower_bound,
                    upper_bound,
                } = new_request
                {
                    assert!(matches!(lower_bound, None));
                    assert!(matches!(upper_bound, Some(ref _key)));
                    assert_eq!(&$any_bound[..], upper_bound.unwrap().as_slice());
                }
            }
        };
    }

    speculate! {
        use std::u8::MAX;

        describe "scan with two bounds" {
            it "normal" {
                assert_scan!(b"last_name", b"name");
            }

            it "zero" {
                assert_scan!([0; 0], [0; 0]);
            }

            it "max length" {
                assert_scan!([1; MAX_KEY_LEN], [MAX - 1; MAX_KEY_LEN]);
            }

            #[should_panic] // should be (None, None)
            it "max range" {
                assert_scan!([0; MAX_KEY_LEN], [MAX; MAX_KEY_LEN]);
            }

            #[should_panic]
            it "overflow" {
                assert_scan!([0; MAX_KEY_LEN + 1], [MAX; MAX_KEY_LEN + 1]);
            }
        }

        describe "scan with one bound" {
            it "normal" {
                assert_scan!(b"last_name");
            }

            it "zero" {
                assert_scan!([0; 0]);
            }

            it "max length" {
                assert_scan!([1; MAX_KEY_LEN]);
            }

            #[should_panic] // should be (None, None)
            it "min lower_bound" {
                assert_scan!([0; MAX_KEY_LEN]);
            }

            #[should_panic] // should be (None, None)
            it "max upper_bound" {
                assert_scan!([MAX; MAX_KEY_LEN]);
            }

            #[should_panic]
            it "overflow" {
                assert_scan!([0; MAX_KEY_LEN + 1]);
            }
        }

        describe "scan with no bound" {
            it "normal" {
                assert_scan!();
            }
        }
    }

    macro_rules! assert_set {
        ($key:expr, $value:expr) => {
            let (new_request, bytes) =
                Request::Set($key[..].to_vec().into(), $value[..].to_vec().into())
                    .transfer_move()
                    .unwrap();
            assert_eq!(5 + $key.len() + $value.len(), bytes);
            assert!(matches!(&new_request, Request::Set(ref _key, ref _value)));
            if let Request::Set(ref key, ref value) = new_request {
                assert_eq!(&$key[..], key.as_slice());
                assert_eq!(&$value[..], value.as_slice());
            }
        };
    }

    speculate! {
        describe "set" {
            it "normal" {
                assert_set!(b"name", b"hexi");
            }

            it "zero" {
                assert_set!([0; 0], [0; 0]);
            }

            it "max length" {
                assert_set!([0; MAX_KEY_LEN], [0; MAX_VALUE_LEN]);
            }

            #[should_panic]
            it "key overflow" {
                assert_set!([0; MAX_KEY_LEN + 1], [0; MAX_VALUE_LEN]);
            }

            #[should_panic]
            it "value overflow" {
                assert_set!([0; MAX_KEY_LEN], [0; MAX_VALUE_LEN + 1]);
            }
        }
    }
}
