use message::{Message, MessageBuilder, Op};
use std::thread;
use std::time::Duration;

/// `Cache`
pub struct Cache;

impl Cache {
    pub fn process(&self, message: Message) -> Message {
        match message.op()  {
            Op::Set => thread::sleep(Duration::from_millis(1000)),
            Op::Get => (),
            Op::Del => (),
        }
       MessageBuilder::default().set_op(Op::Get).set_key(message.key().unwrap().to_vec()).finish().unwrap()
    }
}