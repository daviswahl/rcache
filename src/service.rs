extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate bytes;

use futures::{future, Future};

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::{TcpClient, TcpServer};
use tokio_proto::multiplex::{RequestId, ServerProto, ClientProto, ClientService};
use tokio_service::{Service, NewService};

use bytes::{BytesMut, Buf, BufMut, BigEndian};

use std::{io, str};
use std::net::SocketAddr;

use codec;
pub fn serve<T>(addr: SocketAddr, new_service: T)
where
    T: NewService<Request = codec::Message, Response = codec::Message, Error = io::Error>
        + Send
        + Sync
        + 'static,
{
    TcpServer::new(codec::CacheProto, addr).serve(new_service)
}

pub struct LogService<T> {
    pub inner: T
}

impl<T> Service for LogService<T> where T: Service<Request = codec::Message, Response = codec::Message, Error = io::Error>, T::Future: 'static {
    type Request = codec::Message;
    type Response = codec::Message;
    type Error = io::Error;
    type Future = Box<Future<Item = codec::Message, Error = io::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        println!("Got Request! Op: {:?}, Key: {:?}", req.op, String::from_utf8(req.key.clone()).unwrap());
        Box::new(self.inner.call(req).and_then(|resp| {
            {
                println!("Got Response! Payload: {:?}", String::from_utf8(resp.payload.data.clone()).unwrap());
            }
            Ok(resp)
        }))
    }
}

pub struct CacheService;

impl Service for CacheService
{
    type Request = codec::Message;
    type Response = codec::Message;
    type Error = io::Error;
    type Future = Box<Future<Item = codec::Message, Error = io::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(future::ok(req))
    }
}

impl<T> NewService for LogService<T>
where T: NewService<Request = codec::Message, Response = codec::Message, Error = io::Error>,
      <T::Instance as Service>::Future: 'static
{

    type Request = codec::Message;
    type Response = codec::Message;
    type Error = io::Error;
    type Instance = LogService<T::Instance>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let inner = self.inner.new_service()?;
        Ok(LogService { inner: inner })
    }
}

impl NewService for CacheService
{
    type Request = codec::Message;
    type Response = codec::Message;
    type Error = io::Error;
    type Instance = CacheService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(CacheService)
    }
}
