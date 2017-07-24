use std::convert::TryFrom;
use std::io;

/// `Message`
#[derive(Debug, PartialEq, Clone)]
pub struct Message {
    pub op: Op,
    pub key: Vec<u8>,
    pub payload: Payload,
}

/// `Payload`
#[derive(Debug, PartialEq, Clone)]
pub struct Payload {
    pub type_id: u32,
    pub data: Vec<u8>,
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
