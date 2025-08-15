use std::net::TcpListener;

use bytes::Bytes;
use tcp_server::{error::ServiceError, service::Service, thread_pool::ThreadPool};

fn main() {
    let listener = TcpListener::bind("[::]:8080").unwrap();
    let num = std::thread::available_parallelism().unwrap();
    let thread_pool = ThreadPool::new(num);

    let svc = EchoSvc;

    tcp_server::serve(thread_pool, listener, svc).unwrap();
}

#[derive(Debug, Clone)]
struct EchoSvc;

impl Service for EchoSvc {
    type Response = Bytes;
    type Error = ServiceError;

    fn call(&self, mut req: impl bytes::Buf + 'static) -> Result<Self::Response, Self::Error> {
        let req_body = req.copy_to_bytes(req.remaining());
        println!("got {:?}", req_body);

        if req_body.trim_ascii_end().is_empty() {
            return Err(ServiceError::new(EmptyBody));
        }
        Ok(req_body)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("Got empty body")]
struct EmptyBody;
