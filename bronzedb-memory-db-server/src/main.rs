#[macro_use]
extern crate serde_derive;
use crate::engine_impl::EngineImpl;
use bronzedb_server::Server;
use bronzedb_util::status::Result;
use std::net::TcpListener;

fn main() -> Result<()> {
    env_logger::init();
    let config = conf::Config::new();
    let listener = TcpListener::bind(&config.db_addr)?;
    Server::new(EngineImpl::new()).serve(listener)
}

mod engine_impl;
mod conf;
