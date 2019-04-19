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

fn deal_engine_err<T, E: Into<Error>>(
    stream_ref: &mut TcpStream,
    result: std::result::Result<T, E>,
) -> Result<T> {
    match result {
        Ok(value) => Ok(value),
        Err(err) => {
            Response::Status(EngineError).write_to(stream_ref)?;
            Err(err.into())
        }
    }
}

fn handle_client<T: Engine>(mut stream: TcpStream, mut engine: T) -> Result<()> {
    loop {
        match Request::read_from(&mut stream) {
            Ok(request) => match request {
                Get(key) => {
                    let value = deal_engine_err(&mut stream, engine.get(key.into()))?;
                    match value {
                        Some(data) => Response::SingleValue(data).write_to(&mut stream)?,
                        None => Response::Status(NotFound).write_to(&mut stream)?,
                    };
                }
                Set(key, value) => {
                    deal_engine_err(&mut stream, engine.set(key.into(), value))?;
                    Response::Status(OK).write_to(&mut stream)?;
                }

                Delete(key) => {
                    deal_engine_err(&mut stream, engine.delete(key.into()))?;
                    Response::Status(OK).write_to(&mut stream)?;
                }
                Scan {
                    lower_bound,
                    upper_bound,
                } => {
                    let scanner =
                        deal_engine_err(&mut stream, engine.scan(lower_bound, upper_bound))?;
                    Response::Scanner(scanner.iter()).write_to(&mut stream)?;
                }

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
