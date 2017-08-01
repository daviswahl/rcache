
use futures::Future;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::TcpClient;
use tokio_proto::multiplex::ClientService;
use tokio_service::Service;
use std::net::SocketAddr;
use std::io;

use proto::CacheProto;
use message::{self, Message, Op};

/// A simple client for interacting with `rcache`, intended for debugging, testing, and benchmarking.
/// Can be used as a template for implementing a more robust client.
pub struct Client {
    inner: ClientService<TcpStream, CacheProto>,
}

impl Client {
    pub fn connect(
        addr: &SocketAddr,
        handle: &Handle,
    ) -> impl Future<Item = Client, Error = io::Error> {
        TcpClient::new(CacheProto).connect(addr, handle).map(
            |client_service| Client { inner: client_service },
        )
    }

    pub fn get(&self, key: Vec<u8>) -> Box<Future<Item = Message, Error = io::Error>> {
        let req = message::request(Op::Get, key, None);
        self.call(req)
    }

    pub fn set(&self, key: Vec<u8>, value: Vec<u8>) -> Box<Future<Item = Message, Error = io::Error>> {
        let req = message::request(Op::Set, key, Some(message::payload(1, value)));
        self.call(req)
    }

    pub fn stats(&self) -> Box<Future<Item = Message, Error = io::Error>> {
        let req = message::request(Op::Stats, vec![], None);
        self.call(req)
    }
}

impl Service for Client {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;

    fn call(&self, req: Message) -> Self::Future {
        Box::new(self.inner.call(req))
    }
}
