use codec::CacheCodec;
use message::Message;
use tokio_io::codec::Framed;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::streaming::multiplex::{ClientProto, ServerProto};
use std::io;

/// `CacheProto`
pub struct CacheProto;

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for CacheProto {
    type Request = Message;
    type Response = Message;
    type RequestBody = Message;
    type ResponseBody = Message;
    type Error = io::Error;

    type Transport = Framed<T, CacheCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CacheCodec))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for CacheProto {
    type Request = Message;
    type Response = Message;
    type RequestBody = Message;
    type ResponseBody = Message;
    type Error = io:Error;

    type Transport = Framed<T, CacheCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CacheCodec))
    }
}