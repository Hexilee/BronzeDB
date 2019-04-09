use crate::engine_impl::EngineImpl;
use engine::{Engine, Scanner};
use protocol::request::Request::{self, *};
use protocol::response::Response;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use util::status::StatusCode::{self, *};
use util::status::{Error, Result};
use util::types::{Entry, Key, Value};

fn handle_client<T: Engine>(mut stream: TcpStream, mut engine: T) -> Result<()> {
    loop {
        match Request::read_from(&mut stream)? {
            Get { key } => match engine.get(key.into()) {
                Ok(value) => {
                    Response::SingleValue { status: OK, value }.write_to(&mut stream)?;
                }
                Err(err) => {
                    Response::Status(err.code).write_to(&mut stream)?;
                }
            },
            Set { key, value } => match engine.set(key.into(), value) {
                Ok(_) => {
                    Response::Status(OK).write_to(&mut stream)?;
                }
                Err(err) => {
                    Response::Status(err.code).write_to(&mut stream)?;
                }
            },
            Delete { key } => match engine.delete(key.into()) {
                Ok(value) => {
                    Response::Status(OK).write_to(&mut stream)?;
                }
                Err(err) => {
                    Response::Status(err.code).write_to(&mut stream)?;
                }
            },
            Scan {
                upper_bound,
                lower_bound,
            } => match engine.scan(lower_bound.into(), upper_bound.into()) {},
            Unknown => break Err(Error::new(UnknownAction, "unknown action")),
        }
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    let shared_engine = EngineImpl::new();
    for stream in listener.incoming() {
        let engine = shared_engine.clone();
        spawn(move || handle_client(stream?, engine));
    }
    Ok(())
}

mod engine_impl;
