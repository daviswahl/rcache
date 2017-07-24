use std::convert::TryFrom;
use std::io;

/// `Message`
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    op: Op,
    key: Option<Vec<u8>>,
    payload: Option<Payload>,
}

impl Message {
    pub fn key(&self) -> Option<&[u8]> {
        self.key.as_ref().map(|m| m.as_ref())
    }

    pub fn op(&self) -> Op {
        self.op
    }

    pub fn type_id(&self) -> Option<u32> {
        self.payload.as_ref().map(|p| p.type_id)
    }

    pub fn payload(&self) -> Option<&[u8]> {
        self.payload.as_ref().map(|p| p.data.as_ref())
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
        Self::default()
    }

    pub fn set_op(&mut self, op: Op) -> &mut Self {
        self.op = Some(op);
        self
    }

    pub fn set_key(&mut self, key: Vec<u8>) -> &mut Self {
        if !key.is_empty() {
            self.key = Some(key);
        } else {
            self.key = None;
        }
        self
    }

    pub fn set_payload(&mut self, payload: Vec<u8>) -> &mut Self {
        if payload.is_empty() {
            self.payload = None;
        } else {
            self.payload = Some(payload);
        }
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

        let key = self.key;

        let type_id = if self.payload.is_some() {
            self.type_id.ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "no type_id set",
            ))?
        } else { 0 };

        let payload = self.payload.map(|payload| {
            Payload { data: payload, type_id: type_id }
        });

        Ok(Message {
            op: op,
            key: key,
            payload: payload
        })
    }

    pub fn finish(&self) -> Result<Message, io::Error> {
        let key = self.key.clone();

        let op = self.op.ok_or_else(|| io::Error::new(
            io::ErrorKind::InvalidData,
            "no op set",
        ))?;

        let type_id = if self.payload.is_some() {
            self.type_id.ok_or_else(|| io::Error::new(
                io::ErrorKind::InvalidData,
                "no type_id set",
            ))?
        } else { 0 };

        let payload = self.payload.as_ref().map(|payload| {
            Payload { data: payload.clone(), type_id: type_id }
        });

        Ok(Message {
            op: op,
            key: key,
            payload: payload
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
