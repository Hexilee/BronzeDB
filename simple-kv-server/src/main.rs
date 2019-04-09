use crate::engine_impl::EngineImpl;
use engine::{err, Engine};
use protocol::request::Request::{self, *};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn handle_client<T: Engine>(mut stream: TcpStream, engine: T) -> err::Result<()> {
    loop {
        match Request::read_from(&mut stream)? {
            Get { key } => unimplemented!(),
            Set { key, value } => unimplemented!(),
            Delete { key } => unimplemented!(),
            Scan {
                upper_bound,
                lower_bound,
            } => unimplemented!(),
        }
    }
}

fn main() -> err::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    let shared_engine = EngineImpl::new();
    for stream in listener.incoming() {
        let engine = shared_engine.clone();
        spawn(move || handle_client(stream?, engine));
    }
    Ok(())
}

mod engine_impl;
