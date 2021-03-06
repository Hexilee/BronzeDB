#![cfg(test)]
#![feature(duration_float)]

#[macro_use]
extern crate serde_derive;

use bronzedb_client::{BronzeConnManager, Connection, Pool};
use bronzedb_util::status::Result;
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

#[test]
fn single_thread() -> Result<()> {
    let manager = BronzeConnManager::new(Config::new().db_addr);
    let pool = Pool::builder()
        .max_size(1)
        .build(manager)
        .unwrap();
    const SIZE: u64 = 10_000;
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            pool.get().unwrap().set(key, value)?;
        }
        println!(
            "one connect set: {}/s",
            SIZE as f64 / now.elapsed().as_secs_f64()
        );
    }
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            debug_assert_eq!(value, pool.get().unwrap().get(key)?.unwrap());
        }
        println!(
            "one connect get: {}/s",
            SIZE as f64 / now.elapsed().as_secs_f64()
        );
    }
    {
        let now = Instant::now();
        let mut connect = pool.get().unwrap();
        let scanner = connect.scan(None, None)?;
        let mut counter = 0;
        for item in scanner {
            let (key, value) = item?;
            debug_assert_eq!(value.as_slice(), key.as_slice());
            counter += 1;
        }
        println!(
            "one connect scan: {}/s",
            counter as f64 / now.elapsed().as_secs_f64()
        );
    }
    {
        let now = Instant::now();
        for i in 0..SIZE {
            let key = i.to_string().into_bytes().into();
            pool.get().unwrap().delete(key)?;
        }
        println!(
            "one connect delete: {}/s",
            SIZE as f64 / now.elapsed().as_secs_f64()
        );
    }
    Ok(())
}

#[test]
fn multi_thread() -> Result<()> {
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
            (THREADS * SIZE) as f64 / now.elapsed().as_secs_f64()
        );
    }
    {
        let now = Instant::now();
        let mut handlers = Vec::with_capacity(THREADS as usize);
        for id in 0..THREADS {
            let pool = pool.clone();
            handlers.push(std::thread::spawn(move || -> Result<()> {
                for i in id * SIZE..(id + 1) * SIZE {
                    let value = i.to_string().into_bytes();
                    let key = value.clone().into();
                    debug_assert_eq!(value, pool.get().unwrap().get(key)?.unwrap());
                }
                Ok(())
            }));
        }
        for handler in handlers {
            handler.join().unwrap()?;
        }
        println!(
            "multi connect get: {}/s",
            (THREADS * SIZE) as f64 / now.elapsed().as_secs_f64()
        );
    }
    {
        let now = Instant::now();
        let mut handlers = Vec::with_capacity(THREADS as usize);
        for id in 0..THREADS {
            let pool = pool.clone();
            handlers.push(std::thread::spawn(move || -> Result<usize> {
                let lower_key = (id * SIZE).to_string().into_bytes().into();
                let upper_key = ((id + 1) * SIZE - 1).to_string().into_bytes().into();
                let mut client = pool.get().unwrap();
                let scanner = client.scan(Some(lower_key), Some(upper_key))?;
                let mut counter = 0;
                for item in scanner {
                    let (key, value) = item?;
                    debug_assert_eq!(value.as_slice(), key.as_slice());
                    counter += 1;
                }
                Ok(counter)
            }));
        }

        let mut counter = 0;
        for handler in handlers {
            counter += handler.join().unwrap()?;
        }
        println!(
            "multi connect scan: {}/s",
            counter as f64 / now.elapsed().as_secs_f64()
        );
    }
    {
        let now = Instant::now();
        let mut handlers = Vec::with_capacity(THREADS as usize);
        for id in 0..THREADS {
            let pool = pool.clone();
            handlers.push(std::thread::spawn(move || -> Result<()> {
                for i in id * SIZE..(id + 1) * SIZE {
                    let key = i.to_string().into_bytes().into();
                    pool.get().unwrap().delete(key)?;
                }
                Ok(())
            }));
        }
        for handler in handlers {
            handler.join().unwrap()?;
        }
        println!(
            "multi connect delete: {}/s",
            (THREADS * SIZE) as f64 / now.elapsed().as_secs_f64()
        );
    }
    Ok(())
}
