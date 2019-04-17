#![cfg(test)]

#[macro_use]
extern crate serde_derive;
use client::Client;
use std::net::TcpStream;
use std::time::Instant;

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

fn get_client() -> Client<TcpStream> {
    let conn = TcpStream::connect(&Config::new().db_addr).unwrap();
    Client::new(conn)
}

#[test]
fn one_connect() {
    let mut client = get_client();
    const SIZE: u64 = 100_000;
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            client.set(key, value).unwrap();
        }
        println!("one connect set: {}/s", SIZE / now.elapsed().as_secs());
    }
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            debug_assert_eq!(value, client.get(key).unwrap().unwrap());
        }
        println!("one connect get: {}/s", SIZE / now.elapsed().as_secs());
    }
    {
        let now = Instant::now();
        let (size, scanner) = client.scan(None, None).unwrap();
        for item in scanner {
            let (key, value) = item.unwrap();
            debug_assert_eq!(value.as_slice(), key.as_slice());
        }
        println!("one connect scan: {}/s", size as u64 / now.elapsed().as_secs());
    }
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let key = i.to_string().into_bytes().into();
            client.delete(key).unwrap();
        }
        println!("one connect delete: {}/s", SIZE / now.elapsed().as_secs());
    }
}
