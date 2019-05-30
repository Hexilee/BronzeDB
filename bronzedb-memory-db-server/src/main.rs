use crate::engine_impl::EngineImpl;
use server::Server;
use std::net::TcpListener;
use util::status::Result;

fn main() -> Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    Server::new(EngineImpl::new()).serve(listener)
}

mod engine_impl;
