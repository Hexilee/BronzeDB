use crate::engine_impl::EngineImpl;
use std::net::TcpListener;
use server::Server;
use util::status::Result;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    Server::new(EngineImpl::new()).serve(listener)
}

mod engine_impl;
