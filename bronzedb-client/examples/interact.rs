use bronzedb_client::Connection;
use bronzedb_util::status::{Error, Result, StatusCode};
use std::io::{self, Write};
use std::net::TcpStream;

fn main() -> Result<()> {
    let mut addr = String::new();
    print!("Please enter address: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut addr)?;
    let stream = TcpStream::connect(addr.trim_end())?;
    let mut client = Connection::new(stream);
    loop {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let action: Vec<&str> = buf.trim_end().split(' ').collect();
        debug_assert!(action.len() > 0);
        match action[0].to_lowercase().as_str() {
            "set" => {
                debug_assert!(action.len() == 3);
                client.set(
                    action[1].as_bytes().to_vec().into(),
                    action[2].as_bytes().to_vec(),
                )?;
                println!("OK")
            }

            "get" => {
                debug_assert!(action.len() == 2);
                let value = client.get(action[1].as_bytes().to_vec().into())?;
                match value {
                    Some(data) => println!("{}", String::from_utf8(data).unwrap()),
                    None => println!("<None>"),
                }
            }

            "delete" => {
                debug_assert!(action.len() == 2);
                client.delete(action[1].as_bytes().to_vec().into())?;
                println!("OK")
            }

            "scan" => {
                let mut lower_key = String::new();
                let mut upper_key = String::new();
                print!("lower_bound(default <None>): ");
                io::stdout().flush()?;
                io::stdin().read_line(&mut lower_key)?;
                let lower_bound = match lower_key.trim_end() {
                    "" => None,
                    key => Some(key.as_bytes().to_vec().into()),
                };

                print!("upper_bound(default <None>): ");
                io::stdout().flush()?;
                io::stdin().read_line(&mut upper_key)?;
                let upper_bound = match upper_key.trim_end() {
                    "" => None,
                    key => Some(key.as_bytes().to_vec().into()),
                };
                let scanner = client.scan(lower_bound, upper_bound)?;
                let mut counter = 0;
                for item in scanner {
                    let (key, value) = item?;
                    println!(
                        "{}: {}",
                        String::from_utf8(key.to_vec()).unwrap(),
                        String::from_utf8(value).unwrap()
                    );
                    counter += 1;
                }
                println!("{} items:", counter);
            }
            _ => {
                break Err(Error::new(
                    StatusCode::UnknownAction,
                    format!("unknown action: {}", action[0]),
                ));
            }
        }
    }
}
