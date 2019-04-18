pub extern crate r2d2;
use super::Connection;
use std::net::TcpStream;
use util::status::Error;

pub struct BronzeConnManager {
    db_addr: String,
}

impl BronzeConnManager {
    pub fn new(addr: impl Into<String>) -> Self {
        Self {
            db_addr: addr.into(),
        }
    }
}

impl r2d2::ManageConnection for BronzeConnManager {
    type Connection = Connection<TcpStream>;
    type Error = Error;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let stream = TcpStream::connect(&self.db_addr)?;
        Ok(Self::Connection::new(stream))
    }

    fn is_valid(&self, conn: &mut Self::Connection) -> Result<(), Self::Error> {
        conn.ping()
    }

    fn has_broken(&self, conn: &mut Self::Connection) -> bool {
        match conn.no_response() {
            Ok(()) => false,
            Err(()) => true,
        }
    }
}
