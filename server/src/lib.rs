use engine::Engine;
use log::info;
use protocol::request::Request::{self, *};
use protocol::response::Response;
use std::io::ErrorKind;
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;
use util::status::StatusCode::*;
use util::status::{Error, Result};

pub struct Server<T: Engine> {
    engine: T,
}

impl<T: Engine + Clone + Sync + Send + 'static> Server<T> {
    pub fn new(engine: T) -> Self {
        Self { engine }
    }

    pub fn serve(&mut self, listener: TcpListener) -> Result<()> {
        for stream in listener.incoming() {
            let stream = stream?;
            info!("establish connection from {}", stream.peer_addr()?);
            let engine = self.engine.clone();
            spawn(move || {
                let addr = stream.peer_addr().unwrap();
                handle_client(stream, engine).unwrap();
                info!("close connection from {}", addr);
            });
        }
        Ok(())
    }
}

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

                Ping => {
                    Response::Status(OK).write_to(&mut stream)?;
                }
                NoResponse => continue,
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
