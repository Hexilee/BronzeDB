#![cfg(test)]

#[macro_use]
extern crate serde_derive;

use client::{BronzeConnManager, Connection, Pool};
use std::net::TcpStream;
use std::time::Instant;
use util::status::Result;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    pub db_addr: String,
}

impl Config {
    pub fn new() -> Self {
        let mut settings = config::Config::default();
        settings.merge(config::File::with_name("Settings")).unwrap();
        settings.try_into().unwrap()
    }
}

fn get_client() -> Result<Connection<TcpStream>> {
    let conn = TcpStream::connect(&Config::new().db_addr)?;
    Ok(Connection::new(conn))
}

#[test]
fn one_connect() -> Result<()> {
    let mut client = get_client()?;
    const SIZE: u64 = 100_000;
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            client.set(key, value)?;
        }
        println!("one connect set: {}/s", SIZE / now.elapsed().as_secs());
    }
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            debug_assert_eq!(value, client.get(key)?.unwrap());
        }
        println!("one connect get: {}/s", SIZE / now.elapsed().as_secs());
    }
    {
        let now = Instant::now();
        let (size, scanner) = client.scan(None, None)?;
        for item in scanner {
            let (key, value) = item?;
            debug_assert_eq!(value.as_slice(), key.as_slice());
        }
        println!(
            "one connect scan: {}/s",
            size as u64 / now.elapsed().as_secs()
        );
    }
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let key = i.to_string().into_bytes().into();
            client.delete(key)?;
        }
        println!("one connect delete: {}/s", SIZE / now.elapsed().as_secs());
    }
    Ok(())
}

#[test]
fn multi_connect() -> Result<()> {
    const THREADS: u64 = 1000;
    const SIZE: u64 = 100;
    let manager = BronzeConnManager::new(Config::new().db_addr);
    let pool = Pool::builder()
        .max_size(THREADS as u32)
        .build(manager)
        .unwrap();
    {
        let now = Instant::now();
        let mut handlers = Vec::with_capacity(THREADS as usize);
        for id in 0..THREADS {
            let pool = pool.clone();
            handlers.push(std::thread::spawn(move || -> Result<()> {
                for i in id * SIZE..(id + 1) * SIZE {
                    let value = i.to_string().into_bytes();
                    let key = value.clone().into();
                    pool.get().unwrap().set(key, value)?;
                }
                Ok(())
            }));
        }
        for handler in handlers {
            handler.join().unwrap()?;
        }
        println!(
            "multi connect set: {}/s",
            THREADS * SIZE / now.elapsed().as_secs()
        );
        Ok(())
    }
}
