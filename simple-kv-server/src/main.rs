use engine::status;
use protocol::request::Request::{self, *};
use std::net::{TcpListener, TcpStream};

fn handle_client(mut stream: TcpStream) -> status::Result<()> {
    loop {
        match Request::read_from(&mut stream)? {
            Get { key } => unimplemented!(),
            Set { key, value } => unimplemented!(),
            Delete { key } => unimplemented!(),
            Scan {
                upper_bound,
                lower_bound,
            } => unimplemented!(),
            Unknown => {
                break Err(status::Error::new(
                    status::StatusCode::UnknownAction,
                    "unknown action",
                ));
            }
        }
    }
}

fn main() -> status::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    for stream in listener.incoming() {
        handle_client(stream?);
    }
    Ok(())
}

mod engine_impl;
