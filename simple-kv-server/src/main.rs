use crate::engine_impl::EngineImpl;
use engine::Engine;
use protocol::request::Request::{self, *};

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
