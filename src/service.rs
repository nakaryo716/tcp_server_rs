use std::error::Error;

use bytes::Buf;

pub trait Service: Clone + Send + 'static {
    type Response: Buf;
    type Error: Error;

    fn call(&self, req: impl Buf + 'static) -> Result<Self::Response, Self::Error>;
}
