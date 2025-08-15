use std::{
    error::Error,
    io::{Read, Write},
    net::TcpListener,
};

use bytes::{Buf, Bytes};

use crate::{error::ServiceError, service::Service, thread_pool::ThreadPool};

pub struct Server<I, S>
where
    I: Read + Write,
    S: Service,
    <S as Service>::Response: Buf,
    <S as Service>::Error: Error,
{
    io: I,
    svc: S,
}

impl<I, S> Server<I, S>
where
    I: Read + Write,
    S: Service,
    <S as Service>::Response: Buf,
    <S as Service>::Error: Error + 'static,
{
    pub fn new(io: I, svc: S) -> Self {
        Server { io, svc }
    }

    pub fn serve(&mut self) -> Result<(), ServiceError> {
        let mut buf = Vec::new();
        self.io.read_to_end(&mut buf).map_err(ServiceError::new)?;

        let req = Bytes::from(buf);
        let s = self.svc.call(req).map_err(ServiceError::new)?;
        self.io.write_all(s.chunk()).map_err(ServiceError::new)?;
        Ok(())
    }
}

pub fn serve<S>(
    thread_pool: ThreadPool,
    io: TcpListener,
    svc: S,
) -> Result<(), Box<dyn std::error::Error>>
where
    S: Service<Response = Bytes, Error = ServiceError> + Clone,
{
    loop {
        let svc = svc.clone();
        let (stream, _) = io.accept()?;

        thread_pool.spawn(move || {
            let mut server = Server::new(stream, svc);
            if let Err(e) = server.serve() {
                eprintln!("{:?}", e);
            }
        });
    }
}
