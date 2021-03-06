use super::request::Action::{self, *};
use crate::ext::{ReadKVExt, WriteKVExt};
use bronzedb_util::status::StatusCode::{self, *};
use bronzedb_util::status::{Error, Result};
use bronzedb_util::types::{Entry, Value};
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::io::{Read, Write};

pub enum Response<'a> {
    Status(StatusCode),
    SingleValue(Value),
    Scanner(Box<dyn Iterator<Item = Result<Entry>> + 'a>),
}

impl<'a> Response<'a> {
    pub fn write_to(self, mut writer: impl Write) -> Result<usize> {
        let mut counter = 1usize; // for StatusCode
        match self {
            Response::Status(status) => writer.write_u8(status as u8)?,
            Response::SingleValue(value) => {
                writer.write_u8(OK as u8)?;
                counter += writer.write_value(&value)?;
            }
            Response::Scanner(iter) => {
                writer.write_u8(OK as u8)?;
                for result in iter {
                    match result {
                        Ok((key, value)) => {
                            writer.write_u8(OK as u8)?;
                            counter += 1 + writer.write_key(&key)? + writer.write_value(&value)?;
                        }
                        Err(err) => {
                            writer.write_u8(err.code as u8)?;
                            Err(err)?;
                        }
                    }
                }
                writer.write_u8(Complete as u8)?;
                counter += 1;
            }
        }
        Ok(counter)
    }

    pub fn read_from(reader: &'a mut dyn Read, request_action: Action) -> Result<Self> {
        match reader.read_u8()?.into() {
            OK => match request_action {
                Get => Ok(Response::SingleValue(reader.read_value()?)),
                Delete | Set | Ping => Ok(Response::Status(OK)),
                Scan => Ok(Response::Scanner(Box::new(ReaderIter::new(reader)))),
                Unknown => Err(Error::new(
                    UnknownAction,
                    format!("unknown action: {:?}", request_action),
                )),
                NoResponse => unreachable!(),
            },
            code => Ok(Response::Status(code)),
        }
    }
}

struct ReaderIter<'a> {
    reader: &'a mut dyn Read,
    complete: bool,
    err_occurred: bool,
}

impl<'a> ReaderIter<'a> {
    fn new(reader: &'a mut dyn Read) -> Self {
        Self {
            reader,
            complete: false,
            err_occurred: false,
        }
    }
}

impl Iterator for ReaderIter<'_> {
    type Item = Result<Entry>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.complete || self.err_occurred {
            return None;
        }
        match self.read_entry() {
            Ok(entry) => Some(Ok(entry)),
            Err(ref err) if err.code == Complete => None,
            Err(err) => Some(Err(err)),
        }
    }
}

impl ReaderIter<'_> {
    fn read_entry(&mut self) -> Result<Entry> {
        match self.reader.read_u8()?.into() {
            OK => Ok((self.reader.read_key()?.into(), self.reader.read_value()?)),
            Complete => {
                self.complete = true;
                Err(Error::new(Complete, "complete"))
            }
            code => {
                self.err_occurred = true;
                Err(Error::new(code, "some error"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Response::{self, *};
    use crate::request::Action::*;
    use crate::{MAX_KEY_LEN, MAX_VALUE_LEN};
    use matches::matches;
    use speculate::speculate;
    use std::io::Cursor;
    use bronzedb_util::status::StatusCode::{self, *};
    use bronzedb_util::status::{Error, Result};
    use bronzedb_util::types::Entry;

    macro_rules! transfer_move {
        ($new_resp:ident, $origin_resp:expr, $size:expr, $action:expr) => {
            let mut buffer = Vec::new();
            assert_eq!($size, $origin_resp.write_to(&mut buffer).unwrap());
            let mut reader = Cursor::new(buffer);
            let $new_resp = Response::read_from(&mut reader, $action).unwrap();
        };
    }

    macro_rules! transfer_err {
        ($new_resp:ident, $origin_resp:expr, $action:expr) => {
            let mut buffer = Vec::new();
            assert!(matches!($origin_resp.write_to(&mut buffer), Err(_err)));
            let mut reader = Cursor::new(buffer);
            let $new_resp = Response::read_from(&mut reader, $action).unwrap();
        };
    }

    macro_rules! assert_status_not_ok {
        ($status:expr) => {
            transfer_move!(new_resp, Response::Status($status), 1usize, Get);
            assert!(matches!(new_resp, Status(ref _x)));
            if let Status(status) = new_resp {
                assert_eq!($status, status);
            }
        };
    }

    speculate! {
        describe "status not ok" {
            it "io error" {
                assert_status_not_ok!(IOError);
            }

            it "unknown action" {
                assert_status_not_ok!(UnknownAction);
            }

            it "engine error" {
                assert_status_not_ok!(EngineError);
            }

            it "not found" {
                assert_status_not_ok!(NotFound);
            }
        }
    }

    #[test]
    fn set_ok() {
        transfer_move!(new_resp, Status(StatusCode::OK), 1usize, Set);
        assert!(matches!(new_resp, Status(ref _x)));
        if let Status(code) = new_resp {
            assert_eq!(StatusCode::OK, code);
        }
    }

    #[test]
    fn delete_ok() {
        transfer_move!(new_resp, Status(StatusCode::OK), 1usize, Delete);
        assert!(matches!(new_resp, Status(ref _x)));
        if let Status(code) = new_resp {
            assert_eq!(StatusCode::OK, code);
        }
    }

    macro_rules! assert_get_ok {
        ($value:expr) => {
            transfer_move!(
                new_resp,
                SingleValue($value.to_vec()),
                $value.len() + 3,
                Get
            );
            assert!(matches!(new_resp, SingleValue(_)));
            if let SingleValue(value) = new_resp {
                assert_eq!(&$value[..], value.as_slice());
            }
        };
    }

    speculate! {
        describe "get ok" {
            it "normal" {
                assert_get_ok!(b"Hexi");
            }

            it "zero" {
                assert_get_ok!([0; 0]);
            }

            it "max length" {
                assert_get_ok!([0; MAX_VALUE_LEN]);
            }

            #[should_panic]
            it "overflow" {
                assert_get_ok!([0; MAX_VALUE_LEN + 1]);
            }
        }
    }

    #[test]
    fn scan_ok() {
        let origin_data: Vec<Entry> = vec![
            (b"name"[..].to_vec().into(), b"Hexi"[..].into()),
            (b""[..].to_vec().into(), b""[..].into()),
            (
                [0; MAX_KEY_LEN][..].to_vec().into(),
                [0; MAX_VALUE_LEN][..].into(),
            ),
        ];

        transfer_move!(
            new_resp,
            Scanner(Box::new(origin_data.iter().map(|entry| Ok(entry.clone())))),
            2 + origin_data.len() * 5
                + origin_data
                    .iter()
                    .fold(0, |size, (key, value)| size + key.len() + value.len()),
            Scan
        );
        assert!(matches!(new_resp, Scanner(_)));
        if let Scanner(iter) = new_resp {
            let transferred_data = iter.map(|ret| ret.unwrap()).collect::<Vec<Entry>>();
            assert_eq!(origin_data, transferred_data);
        }
    }

    #[test]
    fn scan_err() {
        let origin_data: Vec<Result<Entry>> = vec![
            Ok((b"name"[..].to_vec().into(), b"Hexi"[..].into())),
            Err(Error::new(StatusCode::IOError, "Some IO Error")),
            Ok((b"last_name"[..].to_vec().into(), b"Lee"[..].into())),
        ];

        transfer_err!(
            new_resp,
            Scanner(Box::new(origin_data.clone().into_iter())),
            Scan
        );
        assert!(matches!(new_resp, Scanner(_)));
        if let Scanner(mut iter) = new_resp {
            assert_eq!(
                origin_data[0].as_ref().unwrap(),
                &iter.next().unwrap().unwrap()
            );
            assert!(matches!(iter.next().unwrap(), Err(_ref)));
            assert!(matches!(iter.next(), None));
        }
    }
}
