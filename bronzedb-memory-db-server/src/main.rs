use crate::engine_impl::EngineImpl;
use bronzedb_server::Server;
use bronzedb_util::status::Result;
use std::net::TcpListener;

fn main() -> Result<()> {
    env_logger::init();
    let listener = TcpListener::bind("127.0.0.1:8088")?;
    Server::new(EngineImpl::new()).serve(listener)
}

mod engine_impl;
