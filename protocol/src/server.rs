use super::request::Action::{self, *};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use engine::status;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};
use std::thread::spawn;

pub trait Service: Send + Sync {
    fn on_set(&mut self, ctx: &mut Context) -> io::Result<()>;
    fn on_get(&self, ctx: &mut Context) -> io::Result<()>;
    fn on_delete(&mut self, ctx: &mut Context) -> io::Result<()>;
    fn on_scan(&self, ctx: &mut Context) -> io::Result<()>;
}

pub trait TcpListenerExt {
    fn serve<S: Service + 'static>(&mut self, service: S) -> status::Result<()>;
}

impl TcpListenerExt for TcpListener {
    fn serve<S: Service + 'static>(&mut self, service: S) -> status::Result<()> {
        let shared_service = Arc::new(RwLock::new(service));
        for stream in self.incoming() {
            let service = shared_service.clone();
            spawn(move || handle_stream(service, stream?));
        }
        Ok(())
    }
}

fn handle_stream<S: Service + 'static>(
    service: Arc<RwLock<S>>,
    mut stream: TcpStream,
) -> status::Result<()> {
    let mut ctx = Context::new(stream);
    loop {
        match ctx.wait_for_request()? {
            Get => service.read()?.on_get(&mut ctx)?,
            Set => service.write()?.on_set(&mut ctx)?,
            Delete => service.write()?.on_delete(&mut ctx)?,
            Scan => service.read()?.on_scan(&mut ctx)?,
            Unknown => {
                break Err(status::Error::new(
                    status::StatusCode::UnknownAction,
                    "unknown action",
                ));
            }
        }
    }
}

pub struct Context {
    stream: TcpStream,
}

impl Context {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream }
    }

    pub fn wait_for_request(&mut self) -> io::Result<Action> {
        Ok(self.stream.read_u8()?.into())
    }
}
