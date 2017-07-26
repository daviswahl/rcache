use std::convert::TryFrom;
use error;

/// `Message`
#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Request(Op, Vec<u8>, Option<Payload>),
    Response(Op, Code, Option<Payload>),
}

pub fn request(op: Op, key: Vec<u8>, payload: Option<Payload>) -> Message {
    Message::Request(op, key, payload)
}

pub fn response(op: Op, code: Code, payload: Option<Payload>) -> Message {
    Message::Response(op, code, payload)
}

impl Message {

    pub fn key(&self) -> Option<&[u8]> {
        match *self {
            Message::Request(_, ref key, _) => Some(key.as_slice()),
            Message::Response(..) => None,
        }
    }

    pub fn op(&self) -> Op {
        match *self {
            Message::Request(op, ..) |
            Message::Response(op, ..) => op,
        }
    }

    pub fn code(&self) -> Code {
        match *self {
            Message::Request(..) => Code::Req,
            Message::Response(_, code, ..) => code,
        }
    }
    pub fn type_id(&self) -> Option<u32> {
        match *self {
            Message::Request(_, _, ref payload) => payload.as_ref().map(|p| p.type_id),
            Message::Response(_, _, ref payload) => payload.as_ref().map(|p| p.type_id),
        }
    }

    pub fn payload(&self) -> Option<&Payload> {
        match *self {
            Message::Request(_, _, ref payload)  |
            Message::Response(_, _, ref payload) => payload.as_ref(),
        }
    }

    pub fn consume_request(self) -> Result<(Vec<u8>, Option<Payload>), error::Error> {
        match self {
            Message::Request(_, key, payload) => Ok((key, payload)),
            Message::Response(..) => Err(error::Error::new(error::ErrorKind::BadMessage, "expected a request, got a response"))
        }
    }
}

/// `Payload`
#[derive(Debug, PartialEq, Clone)]
pub struct Payload {
    type_id: u32,
    data: Vec<u8>,
}

impl Payload {
    pub fn data(&self) -> &[u8] {
        self.data.as_slice()
    }
}

impl From<Payload> for (u32, Vec<u8>) {
    fn from(payload: Payload) -> Self {
        (payload.type_id, payload.data)
    }
}

pub fn payload(type_id: u32, data: Vec<u8>) -> Payload {
    Payload{type_id: type_id, data: data}
}

/// `Op`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Op {
    Set = 0,
    Get = 1,
    Del = 2,
    Stats = 3,
}

impl TryFrom<u8> for Op {
    type Error = error::Error;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(Op::Set),
            1 => Ok(Op::Get),
            2 => Ok(Op::Del),
            3 => Ok(Op::Stats),
            _ => Err(error::Error::new(error::ErrorKind::UnknownOp, "got an unknown op code")),
        }
    }
}

/// `Code`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Code {
    Req = 0,
    Ok = 1,
    Miss = 2,
    Error = 3,
}

impl TryFrom<u8> for Code {
    type Error = error::Error;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(Code::Req),
            1 => Ok(Code::Ok),
            2 => Ok(Code::Miss),
            3 => Ok(Code::Error),
            _ => Err(error::Error::new(error::ErrorKind::InvalidData, "unknown code")),
        }
    }
}
