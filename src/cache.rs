use message::{Message, MessageBuilder, Op};
use std::thread;

/// `Cache`
pub struct Cache;

impl Cache {
    pub fn process(&self, message: Message) -> Message {
        match message.op()  {
            Op::Set => (),
            Op::Get => (),
            Op::Del => (),
        }
       MessageBuilder::default().set_op(Op::Get).set_key(message.key().unwrap().to_vec()).finish().unwrap()
    }
}