use futures::{stream, future, Future, Stream, Sink, IntoFuture};

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use tokio_io::AsyncRead;

use tokio_service::{Service, NewService};

use std::{io, str};
use std::net::SocketAddr;

use message::Message;
use cache;
use codec::CacheCodec;
use futures_cpupool::CpuPool;
use std::thread;
use std::sync::Arc;
use futures::sync::oneshot;

/// `serve`
pub fn serve<T>(addr: SocketAddr, s: T) -> io::Result<()>
where
    T: NewService<Request = Message, Response = Message, Error = io::Error> + Send + Sync + 'static,
{
    let mut core = Core::new()?;
    let handle = core.handle();

    let listener = TcpListener::bind(&addr, &handle)?;

    let connections = listener.incoming();
    let server = connections.for_each(move |(socket, _peer_addr)| {
        let (writer, reader) = socket.framed(CacheCodec).split();
        let service = s.new_service().unwrap();

        let responses = reader.and_then(move |(req_id, msg)| {
            service.call(msg).map(move |resp| (req_id, resp))
        });

        let server = writer.send_all(responses).then(|_| Ok(()));
        handle.spawn(server);
        Ok(())
    });

    core.run(server)
}

/// `CacheService`
pub struct CacheService {
    pub cache: Arc<cache::Cache>,
}

impl Service for CacheService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let (tx, rx) = oneshot::channel();

        let cache = self.cache.clone();
        thread::spawn(move || tx.complete(cache.process(req)));
        rx.map(|msg| msg.unwrap())
            .map_err(|_| io::Error::new(io::ErrorKind::Other, "something's up"))
            .boxed()
    }
}

impl NewService for CacheService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Instance = CacheService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(CacheService { cache: Arc::new(cache::Cache {}) })
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
        //println!("Got Request! {:?}", req);
        Box::new(self.inner.call(req).and_then(|resp| {
         //   println!("Got Response: {:?}", resp);
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
