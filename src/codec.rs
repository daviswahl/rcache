use tokio_io::codec::{Encoder, Decoder, Framed};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_proto::multiplex::{RequestId, ClientProto, ServerProto};
use std::io;
use bytes::{Buf, BufMut, BigEndian, BytesMut};

pub struct CacheCodec;

pub struct CacheProto;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Set = 0,
    Get = 1,
    Del = 2,
}

impl From<u8> for Op {
    fn from(i: u8) -> Self {
        match i {
            0 => Op::Set,
            1 => Op::Get,
            2 => Op::Del,
            _ => Op::Get,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub op: Op,
    pub key: Vec<u8>,
    pub payload: Payload,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Payload {
    pub type_id: u32,
    pub data: Vec<u8>,
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Header {
    op: Op,
    type_id: u32,
    payload_len: u64,
    key_len: u32,
}

/// +-- request id ------+---- op --------+---type id -----+--- payload len ---+---- key len ---
/// |                    |                |                |                   |
/// | u64 (8 bytes)      |  u8 1 byte     | u32 (4 bytes)  |   u64 (8 bytes)   |  u32 (4 bytes)
///
/// +--- key --+-- payload --+
/// |          |             |
/// |   [u8]   |   [u8]      |
///
impl Decoder for CacheCodec {
    type Item = (RequestId, Message);
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<(RequestId, Message)>, io::Error> {
        let header_len = 8 + 1 + 4 + 8 + 4;

        // Check that the header is complete
        if buf.len() < header_len {
            return Ok(None);
        }

        let payload_len = io::Cursor::new(&buf.as_ref()[13..21]).get_u64::<BigEndian>() as usize;
        let key_len = io::Cursor::new(&buf.as_ref()[21..25]).get_u32::<BigEndian>() as usize;

        let message_len = header_len + payload_len + key_len;

        // buffer not ready
        if (buf.len()) < message_len {
            return Ok(None);
        }

        let message = buf.split_to(message_len);

        let request_id = io::Cursor::new(&message[0..8]).get_u64::<BigEndian>();
        let op = io::Cursor::new(&message[8..9]).get_u8();
        let type_id = io::Cursor::new(&message[9..13]).get_u32::<BigEndian>();
        let key = &message[header_len..header_len + key_len];
        let data = &message[header_len + key_len..header_len + key_len + payload_len];

        Ok(Some((
            request_id as RequestId,
            Message {
                op: Op::from(op),
                key: key.to_owned(),
                payload: Payload {
                    type_id: type_id,
                    data: data.to_owned(),
                },
            },
        )))
    }
}

impl Encoder for CacheCodec {
    type Item = (RequestId, Message);
    type Error = io::Error;

    fn encode(&mut self, msg: (RequestId, Message), buf: &mut BytesMut) -> io::Result<()> {
        let (request_id, msg) = msg;
        let min_size = 8 + 1 + 4 + 8 + 4 + msg.key.len() + msg.payload.data.len();
        buf.reserve(min_size);
        buf.put_u64::<BigEndian>(request_id as u64);
        buf.put_u8(msg.op as u8);
        buf.put_u32::<BigEndian>(msg.payload.type_id as u32);
        buf.put_u64::<BigEndian>(msg.payload.data.len() as u64);
        buf.put_u32::<BigEndian>(msg.key.len() as u32);
        buf.put_slice(msg.key.as_ref());
        buf.put_slice(msg.payload.data.as_ref());
        Ok(())
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ClientProto<T> for CacheProto {
    type Request = Message;
    type Response = Message;

    type Transport = Framed<T, CacheCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CacheCodec))
    }
}

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for CacheProto {
    type Request = Message;
    type Response = Message;

    type Transport = Framed<T, CacheCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;

    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(CacheCodec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn assert_sizes() {
        use std::mem;
        assert_eq!(8, mem::size_of::<u64>());
        assert_eq!(1, mem::size_of::<Op>());
        assert_eq!(2, mem::size_of::<u16>());
        assert_eq!(8, mem::size_of::<usize>());
        assert_eq!(4, mem::size_of::<u32>());
    }
    #[test]
    fn test() {
        let msg = Message {
            op: Op::Get,
            key: "foo".into(),
            payload: Payload {
                type_id: 2,
                data: "asdfoiuasdf".to_owned().into_bytes(),
            },
        };


        println!("message: {:?}", msg);
        let req_id = 123 as RequestId;
        let mut buf = BytesMut::new();
        let mut codec = CacheCodec;

        codec.encode((req_id, msg.clone()), &mut buf);
        let (decoded_req, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded_req, req_id);
        assert_eq!(decoded_message, msg);
    }
}
