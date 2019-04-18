use protocol::request::Action::{Delete, Get, Ping, Scan, Set};
use protocol::request::Request;
use protocol::response::Response::{self, *};
use std::io::{Read, Write};
use util::status::StatusCode::*;
use util::status::{Error, Result};
use util::types::{Entry, Key, Value};

pub struct Connection<T: Read + Write> {
    inner: T,
}

impl<T: Read + Write> Connection<T> {
    pub fn new(connection: T) -> Self {
        Self { inner: connection }
    }

    pub fn set(&mut self, key: Key, value: Value) -> Result<()> {
        Request::Set(key, value).write_to(&mut self.inner)?;
        match Response::read_from(&mut self.inner, Set)? {
            Status(status) => match status {
                OK => Ok(()),
                code => Err(Error::new(code, "set request error")),
            },
            _ => unreachable!(),
        }
    }

    pub fn delete(&mut self, key: Key) -> Result<()> {
        Request::Delete(key).write_to(&mut self.inner)?;
        match Response::read_from(&mut self.inner, Delete)? {
            Status(status) => match status {
                OK => Ok(()),
                code => Err(Error::new(code, "delete request error")),
            },
            _ => unreachable!(),
        }
    }

    pub fn get(&mut self, key: Key) -> Result<Option<Value>> {
        Request::Get(key).write_to(&mut self.inner)?;
        match Response::read_from(&mut self.inner, Get)? {
            Status(status) => match status {
                NotFound => Ok(None),
                code => Err(Error::new(code, "get request error")),
            },
            SingleValue { status: _, value } => Ok(Some(value)),
            _ => unreachable!(),
        }
    }

    pub fn scan(
        &mut self,
        lower_bound: Option<Key>,
        upper_bound: Option<Key>,
    ) -> Result<(usize, Box<dyn Iterator<Item = Result<Entry>> + '_>)> {
        Request::Scan {
            lower_bound,
            upper_bound,
        }
        .write_to(&mut self.inner)?;
        match Response::read_from(&mut self.inner, Scan)? {
            Status(status) => Err(Error::new(status, "scan request error")),

            MultiKV {
                status: _,
                size,
                iter,
            } => Ok((size, iter)),
            _ => unreachable!(),
        }
    }

    pub fn ping(&mut self) -> Result<()> {
        Request::Ping.write_to(&mut self.inner)?;
        match Response::read_from(&mut self.inner, Ping)? {
            Status(OK) => Ok(()),
            Status(status) => Err(Error::new(status, "ping error")),
            _ => unreachable!(),
        }
    }

    pub fn no_response(&mut self) -> Result<()> {
        Request::NoResponse.write_to(&mut self.inner)?;
        Ok(())
    }
}
