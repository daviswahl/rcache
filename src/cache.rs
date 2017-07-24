use message::{Message, MessageBuilder, Op};
use tokio_core::reactor::{Core, Remote};
use std::error::Error;
use futures::sync::oneshot;
use futures_cpupool::CpuPool;
use std::thread;
use std::sync::Arc;
use futures::{Future, future, BoxFuture};
use rand::{self, Rng};
use std::time::Duration;
use std::io;

/// `Cache`
pub struct Cache {
    pool: CpuPool,
    core: Core,
}

impl Cache {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Cache { pool: CpuPool::new_num_cpus(), core: Core::new()? })
    }
}

impl Cache {
    pub fn process(&self, message: Message) -> BoxFuture<Message, io::Error> {
        let (snd, rcv) = oneshot::channel();

        let result = move||match message.op() {
            Op::Set => {
                let duration = *rand::thread_rng()
                    .choose(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
                    .unwrap();
                println!("{:?} sleeping for {}", message.type_id(), duration * 50);
                thread::sleep(Duration::from_millis(duration * 50));
                snd.send(message)
            }
            Op::Get => snd.send(message),
            Op::Del => snd.send(message),
        };
        thread::spawn(|| result());

        rcv.map_err(|e| io::Error::new(io::ErrorKind::Other, e.description())).boxed()
    }
}
