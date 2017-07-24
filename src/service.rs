extern crate futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate tokio_proto;
extern crate tokio_service;
extern crate bytes;

use futures::{future, Future};

use tokio_proto::TcpServer;
use tokio_service::{Service, NewService};

use std::{io, str};
use std::net::SocketAddr;

use codec;
use proto::CacheProto;
use message::Message;

/// `serve`
pub fn serve<T>(addr: SocketAddr, new_service: T)
where
    T: NewService<Request = Message, Response = Message, Error = io::Error> + Send + Sync + 'static,
{
    TcpServer::new(CacheProto, addr).serve(new_service)
}

/// `CacheService`
pub struct CacheService;

impl Service for CacheService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        Box::new(future::ok(req))
    }
}


impl NewService for CacheService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Instance = CacheService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(CacheService)
    }
}

/// `LogService`
pub struct LogService<T> {
    pub inner: T,
}

impl<T> Service for LogService<T>
    where T: Service<Request = Message, Response = Message, Error = io::Error>,
          T::Future: 'static {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        println!("Got Request! {:?}", req);
        Box::new(self.inner.call(req).and_then(|resp| {
            println!("Got Response: {:?}", resp);
            Ok(resp)
        }))
    }
}

impl<T> NewService for LogService<T>
where
    T: NewService<
        Request = Message,
        Response = Message,
        Error = io::Error,
    >,
    <T::Instance as Service>::Future: 'static,
{
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Instance = LogService<T::Instance>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let inner = self.inner.new_service()?;
        Ok(LogService { inner: inner })
    }
}
