use tokio_io::codec::{Encoder, Decoder};
use tokio_proto::multiplex::RequestId;
use std::io;
use std::convert::TryFrom;
use bytes::{Buf, BufMut, BigEndian, BytesMut};
use message::{self, Message, Op, Code};


static HEADER_LEN: usize = 8 + 1 + 1 + 8 + 4;
/// +-- request id ------+- code ---------+----op --+--- payload len ---+---- key len ---
/// |                    |                |         |                   |
/// | u64 (8 bytes)      | u8, 0 = req    |   u8    |  u64 (8 bytes)    |  u32 (4 bytes)
/// |                    |                |         |                   |
/// +--------------------+----------------+---------+-------------------+----------------
///
/// +--- key --+---type id --+-- payload --+
/// |          |             |             |
/// |   [u8]   |   u32       |    [u8]     |
/// |          |             |             |
/// +----------+-------------+-------------+

/// `CacheCodec`
pub struct CacheCodec;

impl Encoder for CacheCodec {
    type Item = (RequestId, Message);
    type Error = io::Error;

    fn encode(&mut self, msg: (RequestId, Message), buf: &mut BytesMut) -> io::Result<()> {
        let (request_id, msg) = msg;

        let key = msg.key().unwrap_or_else(|| &[]);
        let payload = msg.payload().map(|p| p.data()).unwrap_or_else(|| &[]);
        let type_id = msg.type_id().unwrap_or(0 as u32);

        let type_id_len = if payload.is_empty() {
            0
        } else {
            4
        };

        let payload_len = payload.len();

        let min_size = HEADER_LEN + key.len() + payload_len + type_id_len;
        buf.reserve(min_size);

        buf.put_u64::<BigEndian>(request_id as u64);
        buf.put_u8(msg.code() as u8);
        buf.put_u8(msg.op() as u8);
        buf.put_u64::<BigEndian>(payload_len as u64);
        buf.put_u32::<BigEndian>(key.len() as u32);
        buf.put_slice(key);

        if payload_len > 0 {
            buf.put_u32::<BigEndian>(type_id);
            buf.put_slice(payload);
        }

        Ok(())
    }
}

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
        let payload_len = io::Cursor::new(&buf.as_ref()[10..18]).get_u64::<BigEndian>() as usize;
        let key_len = io::Cursor::new(&buf.as_ref()[18..22]).get_u32::<BigEndian>() as usize;

        let payload_len = if payload_len > 0 { payload_len + 4 } else { 0 };

        let msg_len = HEADER_LEN + payload_len + key_len;
        // buffer not ready
        if (buf.len()) < msg_len {
            return Ok(None);
        }

        let msg = buf.split_to(msg_len);
        let mut cursor = io::Cursor::new(msg);
        let request_id = cursor.get_u64::<BigEndian>();
        let code = cursor.get_u8();
        let op = cursor.get_u8();

        // Skip the payload_len and key_len as they've been read already.
        cursor.advance(12);

        // read the key
        let mut key = Vec::with_capacity(key_len);
        key.resize(key_len, 0);
        cursor.copy_to_slice(&mut key);

        let type_id = if payload_len > 0 { cursor.get_u32::<BigEndian>() } else { 0 };

        let payload = if payload_len > 0 {
            Some(message::payload(type_id, cursor.collect()))
        } else {
            None
        };

        let msg = if code == 0 {
            message::request(Op::try_from(op)?, key.to_vec(), payload)
        } else {
            message::response(Op::try_from(op)?, Code::try_from(code)?, payload)
        };

        Ok(Some((request_id as RequestId, msg)))
    }
}


#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_request() {
        let msg = message::request(
            Op::Get,
            "foo".into(),
            Some(message::payload(3, "123124125".into())),
        );
        let req_id = 123 as RequestId;
        let mut buf = BytesMut::new();
        let mut codec = CacheCodec;

        codec.encode((req_id, msg.clone()), &mut buf).unwrap();
        let (decoded_req, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded_req, req_id);
        assert_eq!(decoded_message, msg);
    }

    #[test]
    fn test_response() {
        let msg = message::response(
            Op::Get,
            Code::Ok,
            Some(message::payload(3, "123124125".into())),
        );
        let req_id = 123 as RequestId;
        let mut buf = BytesMut::new();
        let mut codec = CacheCodec;

        codec.encode((req_id, msg.clone()), &mut buf).unwrap();
        let (decoded_req, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded_req, req_id);
        assert_eq!(decoded_message, msg);
    }

    #[test]
    fn test_request_no_payload() {
        let msg = message::request(Op::Get, "foo".into(), None);
        let req_id = 123 as RequestId;
        let mut buf = BytesMut::new();
        let mut codec = CacheCodec;

        codec.encode((req_id, msg.clone()), &mut buf).unwrap();
        let (decoded_req, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded_req, req_id);
        assert_eq!(decoded_message, msg);
    }
    #[test]
    fn test_response_no_payload() {
        let msg = Message::Response(Op::Set, Code::Ok, None);


        let req_id = 123 as RequestId;
        let mut buf = BytesMut::new();
        let mut codec = CacheCodec;

        codec.encode((req_id, msg.clone()), &mut buf).unwrap();
        let (decoded_req, decoded_message) = codec.decode(&mut buf).unwrap().unwrap();

        assert_eq!(decoded_req, req_id);
        assert_eq!(decoded_message, msg);
    }
}
