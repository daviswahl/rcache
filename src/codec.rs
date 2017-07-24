use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::multiplex::RequestId;
use std::io;
use std::convert::TryFrom;
use bytes::{Buf, BufMut, BigEndian, BytesMut};
use message::{Message, MessageBuilder, Op};

static HEADER_LEN: usize = 25;
/// +-- request id ------+---- op --------+---type id -----+--- payload len ---+---- key len ---
/// |                    |                |                |                   |
/// | u64 (8 bytes)      |  u8 1 byte     | u32 (4 bytes)  |   u64 (8 bytes)   |  u32 (4 bytes)
/// |                    |                |                |                   |
/// +--------------------+----------------+----------------+-------------------+----------------
///
/// +--- key --+-- payload --+
/// |          |             |
/// |   [u8]   |   [u8]      |
/// |          |             |
/// +----------+-------------+

/// `CacheCodec`
pub struct CacheCodec;
impl Decoder for CacheCodec {
    type Item = (RequestId, Message);
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<(RequestId, Message)>, io::Error> {
        // Check that the header is complete
        if buf.len() < HEADER_LEN {
            return Ok(None);
        }

        // TODO: wrapping each slice in a Cursor just to get access to the .get_* methods
        // seems like a lot of overhead, but I could be wrong.
        let payload_len = io::Cursor::new(&buf.as_ref()[13..21]).get_u64::<BigEndian>() as usize;
        let key_len = io::Cursor::new(&buf.as_ref()[21..25]).get_u32::<BigEndian>() as usize;

        let message_len = HEADER_LEN + payload_len + key_len;
        // buffer not ready
        if (buf.len()) < message_len {
            return Ok(None);
        }

        let message = buf.split_to(message_len);

        let request_id = io::Cursor::new(&message[0..8]).get_u64::<BigEndian>();
        let op = io::Cursor::new(&message[8..9]).get_u8();
        let type_id = io::Cursor::new(&message[9..13]).get_u32::<BigEndian>();
        let key = &message[HEADER_LEN..HEADER_LEN + key_len];
        let data = &message[HEADER_LEN + key_len..HEADER_LEN + key_len + payload_len];

        let mut msg = MessageBuilder::new();
        {
            msg.set_op(Op::try_from(op)?)
            .set_key(key.to_owned())
            .set_type_id(type_id)
            .set_payload(data.to_owned());
        }

        Ok(Some((request_id as RequestId, msg.into_message()?)))
    }
}

impl Encoder for CacheCodec {
    type Item = (RequestId, Message);
    type Error = io::Error;

    fn encode(&mut self, msg: (RequestId, Message), buf: &mut BytesMut) -> io::Result<()> {
        let (request_id, msg) = msg;
        let min_size = 8 + 1 + 4 + 8 + 4 + msg.key().len() + msg.payload().len();
        buf.reserve(min_size);
        buf.put_u64::<BigEndian>(request_id as u64);
        buf.put_u8(msg.op() as u8);
        buf.put_u32::<BigEndian>(msg.type_id() as u32);
        buf.put_u64::<BigEndian>(msg.payload().len() as u64);
        buf.put_u32::<BigEndian>(msg.key().len() as u32);
        buf.put_slice(msg.key());
        buf.put_slice(msg.payload());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use message::Payload;
    use message::Op;

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
        let msg = MessageBuilder::new()
            .set_op(Op::Get).set_key("foo".into()).set_type_id(3).set_payload("123091823".into())
            .finish().unwrap();


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
