
use futures::Future;
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::TcpClient;
use tokio_proto::multiplex::ClientService;
use tokio_service::Service;
use std::net::SocketAddr;
use std::io;

use codec;
use service;
use message::Message;

/// `Client`
pub struct Client {
    inner: service::LogService<ClientService<TcpStream, codec::CacheProto>>,
}

impl Client {
    pub fn connect(
        addr: &SocketAddr,
        handle: &Handle,
    ) -> impl Future<Item = Client, Error = io::Error> {
        TcpClient::new(codec::CacheProto)
            .connect(addr, handle)
            .map(|client_service| {
                Client { inner: service::LogService { inner: client_service } }
            })
    }
}

impl Service for Client {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;

    fn call(&self, req: Message) -> Self::Future {
        self.inner.call(req)
    }
}
