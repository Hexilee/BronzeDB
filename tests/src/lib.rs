#![cfg(test)]

#[macro_use]
extern crate serde_derive;

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

mod set {
    use super::Config;
    use client::Client;
    use std::net::TcpStream;
    use std::time::Instant;

    #[test]
    fn one_connect() {
        let conn = TcpStream::connect(&Config::new().db_addr).unwrap();
        let mut client = Client::new(conn);
        let now = Instant::now();
        for i in 0..100_000 {
            let value = i.to_string().into_bytes();
            let key = value.clone().into();
            client.set(key, value).unwrap();
        }
        println!("one connect set: {}/s", 100_000 / now.elapsed().as_secs());
    }
}
