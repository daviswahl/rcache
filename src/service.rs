use futures::{stream, future, Future, Stream, Sink, IntoFuture};

use tokio_core::reactor::Core;
use tokio_core::net::TcpListener;

use tokio_io::AsyncRead;

use tokio_service::{Service, NewService};

use std::{io, str};
use std::net::SocketAddr;

use message::{Message, MessageBuilder, Op};
use cache;
use codec::CacheCodec;
use futures_cpupool::CpuPool;
use std::thread;
use std::sync::Arc;
use futures::sync::oneshot;
use stats::Stats;

/// `serve`
pub fn serve<T>(addr: SocketAddr, s: T) -> io::Result<()>
where
    T: NewService<Request = Message, Response = Message, Error = io::Error> + 'static,
    <T::Instance as Service>::Future: 'static,
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
        self.cache.process(req)
    }
}

impl NewService for CacheService {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Instance = CacheService;

    fn new_service(&self) -> io::Result<Self::Instance> {
        Ok(CacheService { cache: self.cache.clone() })
    }
}

/// `StatService`
pub struct StatService<T> {
    pub inner: T,
    pub stats: Arc<Stats>,
}

impl<T> Service for StatService<T>
    where T: Service<Request = Message, Response = Message, Error = io::Error>,
          T::Future: 'static {
    type Request = Message;
    type Response = Message;
    type Error = io::Error;
    type Future = Box<Future<Item = Message, Error = io::Error>>;

    fn call(&self, req: Self::Request) -> Self::Future {
        match req.op() {
            Op::Stats => {
                let payload = self.stats.get_stats();
                let mut mb = MessageBuilder::default();
                {
                    mb.set_op(Op::Stats).set_payload(payload.into_bytes())
                        .set_type_id(1).set_key("".to_owned().into_bytes());
                }
                Box::new(future::done(mb.into_message()))
            }
            _ => {
                let stats = self.stats.clone();
                Box::new(self.inner.call(req).and_then(move|resp|{
                    stats.incr_handled();
                    Ok(resp)
                }))
            }
        }
    }
}

impl<T> NewService for StatService<T>
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
    type Instance = StatService<T::Instance>;

    fn new_service(&self) -> io::Result<Self::Instance> {
        let inner = self.inner.new_service()?;
        Ok(StatService {
            inner: inner,
            stats: self.stats.clone(),
        })
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
        Box::new(self.inner.call(req).and_then(|resp| {
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
