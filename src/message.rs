use std::convert::TryFrom;
use error;
use std::io;

/// `Message`
#[derive(Debug, PartialEq, Clone)]
pub enum Message {
    Request(Op, Vec<u8>, Option<Payload>),
    Response(Op, Code, Option<Payload>),
}

impl Message {
    pub fn key(&self) -> Option<&[u8]> {
        match self {
            &Message::Request(_, ref key, _) => Some(key.as_slice()),
            &Message::Response(..) => None,
        }
    }

    pub fn op(&self) -> Op {
        match self {
            &Message::Request(op, ..) => op,
            &Message::Response(op, ..) => op,
        }
    }

    pub fn code(&self) -> Code {
        match self {
            &Message::Request(..) => Code::Req,
            &Message::Response(_, code, ..) => code,
        }
    }
    pub fn type_id(&self) -> Option<u32> {
        match self {
            &Message::Request(_, _, ref payload) => payload.as_ref().map(|p| p.type_id),
            &Message::Response(_, _, ref payload) => payload.as_ref().map(|p| p.type_id),
        }
    }

    pub fn payload(&self) -> Option<&Payload> {
        match self {
            &Message::Request(_, _, ref payload) => payload.as_ref(),
            &Message::Response(_, _, ref payload) => payload.as_ref(),
        }
    }

    pub fn consume(self) -> (Option<Vec<u8>>, Option<Payload>) {
        match self {
            Message::Request(_, key, payload) => (Some(key), payload),
            Message::Response(_, key, payload) => (None, payload),
        }
    }
}

/// `MessageBuilder`
#[derive(Debug, PartialEq, Clone, Default)]
pub struct MessageBuilder {
    op: Option<Op>,
    key: Option<Vec<u8>>,
    type_id: Option<u32>,
    payload: Option<Vec<u8>>,
    code: Option<Code>,
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

    pub fn set_code(&mut self, code: Code) -> &mut Self {
        self.code = Some(code);
        self
    }

    pub fn into_request(self) -> Result<Message, error::Error> {
        let payload = if let Some(payload) = self.payload {
            if let Some(type_id) = self.type_id {
                Some(Payload {
                    type_id: type_id,
                    data: payload,
                })
            } else {
                return Err(error::Error::new(error::ErrorKind::InvalidData, "no type_id set"));
            }
        } else {
            None
        };

        let op = self.op.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no op set")
        })?;
        let key = self.key.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no key set")
        })?;
        Ok(Message::Request(op, key, payload))
    }

    pub fn into_response(self) -> Result<Message, error::Error> {
        let payload = if let Some(payload) = self.payload {
            if let Some(type_id) = self.type_id {
                Some(Payload {
                    type_id: type_id,
                    data: payload,
                })
            } else {
                return Err(error::Error::new(
                    error::ErrorKind::InvalidData,
                    "payload given but no type_id set",
                ));
            }
        } else {
            None
        };
        let op = self.op.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no op set")
        })?;
        let code = self.code.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no code set")
        })?;
        Ok(Message::Response(op, code, payload))
    }

    pub fn request(&self) -> Result<Message, error::Error> {
        let payload = if let Some(payload) = self.payload.clone() {
            if let Some(type_id) = self.type_id {
                Some(Payload {
                    type_id: type_id,
                    data: payload,
                })
            } else {
                return Err(error::Error::new(
                    error::ErrorKind::InvalidData,
                    "payload given but no type_id set",
                ));
            }
        } else {
            None
        };

        let op = self.op.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no op set")
        })?;
        let key = self.key.clone().ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no key set")
        })?;
        Ok(Message::Request(
            op,
            key,
            payload
        ))
    }

    pub fn response(&self) -> Result<Message, error::Error> {
        let payload = if let Some(payload) = self.payload.clone() {
            if let Some(type_id) = self.type_id {
                Some(Payload {
                    type_id: type_id,
                    data: payload,
                })
            } else {
                return Err(error::Error::new(
                    error::ErrorKind::InvalidData,
                    "payload given but no type_id set",
                ));
            }
        } else {
            None
        };
        let op = self.op.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no op set")
        })?;
        let code = self.code.ok_or_else(|| {
            error::Error::new(error::ErrorKind::InvalidData, "no code set")
        })?;
        Ok(Message::Response(
            op,
            code,
            payload
        ))
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
            _ => Err(error::Error::new(error::ErrorKind::InvalidData, "unknown op")),
        }
    }
}

/// `Code`
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Code {
    Req = 0,
    Ok = 1,
    Miss = 2,
}

impl TryFrom<u8> for Code {
    type Error = error::Error;

    fn try_from(i: u8) -> Result<Self, Self::Error> {
        match i {
            0 => Ok(Code::Req),
            1 => Ok(Code::Ok),
            2 => Ok(Code::Miss),
            _ => Err(error::Error::new(error::ErrorKind::InvalidData, "unknown code")),
        }
    }
}
