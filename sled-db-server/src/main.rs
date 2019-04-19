#[macro_use]
extern crate serde_derive;
use crate::engine_impl::EngineImpl;
use server::Server;
use std::net::TcpListener;
use util::status::Result;

fn main() -> Result<()> {
    env_logger::init();
    let config = conf::Config::new();
    let listener = TcpListener::bind(&config.db_addr)?;
    Server::new(EngineImpl::new(&config.db_path)).serve(listener)
}

mod conf;
mod engine_impl;
