use message::{Message, MessageBuilder, Op};
use std::thread;
use rand::{self, Rng};
use std::time::Duration;
use std::io;

/// `Cache`
pub struct Cache;

impl Cache {
    pub fn process(&self, message: Message) -> Result<Message, io::Error> {
        match message.op()  {
            Op::Set => {
                //let duration = *rand::thread_rng().choose(&[0,1,2,3,4,5,6,7,8,9,10]).unwrap();
                //println!("{:?} sleeping for {}", message.type_id(), duration * 50);
                //thread::sleep(Duration::from_millis(duration * 50))
            },
            Op::Get => (),
            Op::Del => (),
        }
       Ok(message)
    }
}