use crate::engine_impl::EngineImpl;
use engine::Engine;
use protocol::request::Request::{self, *};
use protocol::response::Response;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use util::status::StatusCode::*;
use util::status::{Error, Result};

fn handle_client<T: Engine>(mut stream: TcpStream, mut engine: T) -> Result<()> {
    loop {
        match Request::read_from(&mut stream) {
            Ok(request) => match request {
                Get(key) => match engine.get(key.into()) {
                    Ok(value) => {
                        Response::SingleValue { status: OK, value }.write_to(&mut stream)?;
                    }
                    Err(err) => {
                        Response::Status(err.code).write_to(&mut stream)?;
                    }
                },
                Set(key, value) => match engine.set(key.into(), value) {
                    Ok(_) => {
                        Response::Status(OK).write_to(&mut stream)?;
                    }
                    Err(err) => {
                        Response::Status(err.code).write_to(&mut stream)?;
                    }
                },
                Delete(key) => match engine.delete(key.into()) {
                    Ok(_) => {
                        Response::Status(OK).write_to(&mut stream)?;
                    }
                    Err(err) => {
                        Response::Status(err.code).write_to(&mut stream)?;
                    }
                },
                Scan {
                    lower_bound,
                    upper_bound,
                } => match engine.scan(lower_bound, upper_bound) {
                    Ok(scanner) => {
                        Response::MultiKV {
                            status: OK,
                            size: scanner.size(),
                            iter: scanner.iter(),
                        }
                        .write_to(&mut stream)?;
                    }
                    Err(err) => {
                        Response::Status(err.code).write_to(&mut stream)?;
                    }
                },
                Unknown => {
                    Response::Status(UnknownAction).write_to(&mut stream)?;
                    break Err(Error::new(UnknownAction, "unknown action"));
                }
            },

            Err(ref err) if err.kind() == ErrorKind::UnexpectedEof => break Ok(()), // shutdown
            Err(err) => break Err(err.into()),
        }
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    let shared_engine = EngineImpl::new();
    for stream in listener.incoming() {
        let engine = shared_engine.clone();
        let stream = stream?;
        println!("establish connection from {}", stream.peer_addr()?);
        spawn(move || {
            let addr = stream.peer_addr().unwrap();
            handle_client(stream, engine).unwrap();
            println!("close connection from {}", addr);
        });
    }
    Ok(())
}

mod engine_impl;
