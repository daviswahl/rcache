use std::convert::TryFrom;
use std::io;

/// `Message`
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    op: Op,
    key: Vec<u8>,
    payload: Payload,
}

impl Message {
    pub fn key(&self) -> &[u8] {
        self.key.as_ref()
    }

    pub fn op(&self) -> Op {
        self.op
    }

    pub fn type_id(&self) -> u32 {
        self.payload.type_id
    }

    pub fn payload(&self) -> &[u8] {
        self.payload.data.as_ref()
    }
}

/// `MessageBuilder`
#[derive(Debug, PartialEq, Clone, Default)]
pub struct MessageBuilder {
    op: Option<Op>,
    key: Option<Vec<u8>>,
    type_id: Option<u32>,
    payload: Option<Vec<u8>>,
}

impl MessageBuilder {
    pub fn new() -> Self {
        MessageBuilder {
            op: None,
            key: None,
            payload: None,
            type_id: None,
        }
    }

    pub fn set_op(&mut self, op: Op) -> &mut Self {
        self.op = Some(op);
        self
    }

    pub fn set_key(&mut self, key: Vec<u8>) -> &mut Self {
        self.key = Some(key);
        self
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) -> &mut Self {
        self.payload = Some(payload);
        self
    }

    pub fn set_type_id(&mut self, type_id: u32) -> &mut Self {
        self.type_id = Some(type_id);
        self
    }

    pub fn into_message(self) -> Result<Message, io::Error> {
        let op = self.op.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no op set",
        ))?;
        let payload = self.payload.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no payload set",
        ))?;
        let key = self.key.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no key set",
        ))?;
        let type_id = self.type_id.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no type_id set",
        ))?;

        Ok(Message {
            op: op,
            key: key,
            payload: Payload {
                type_id: type_id,
                data: payload,
            },
        })
    }

    pub fn finish(&self) -> Result<Message, io::Error> {
        let op = self.op.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no op set",
        ))?;
        let payload = self.payload.clone().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no payload set",
        ))?;
        let key = self.key.clone().ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no key set",
        ))?;
        let type_id = self.type_id.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no type_id set",
        ))?;

        Ok(Message {
            op: op,
            key: key,
            payload: Payload {
                type_id: type_id,
                data: payload,
            },
        })
    }
}

/// `Payload`
#[derive(Debug, PartialEq, Clone)]
pub struct Payload {
    type_id: u32,
    data: Vec<u8>,
}

/// `Op`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Set = 0,
    Get = 1,
    Del = 2,
}

impl TryFrom<u8> for Op {
    type Error = io::Error;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(Op::Set),
            1 => Ok(Op::Get),
            2 => Ok(Op::Del),
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "unknown op")),
        }
    }
}
