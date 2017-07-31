use message::{self, Message, Op, Code, Payload};
use tokio_core::reactor::Core;
use std::error::Error;
use futures::sync::oneshot::Sender;
use futures_cpupool::CpuPool;
use std::sync::{Arc, Mutex, MutexGuard};
use futures::future;
use std::io;
use std::thread;
use error;
use lru_cache::LruCache;
use deque::{self, Worker, Stealer, Stolen};


type Store = LruCache<Vec<u8>, Payload>;
type Work = (Sender<Message>, Message);

/// `Cache`
pub struct Cache {
    pool: CpuPool,
    core: Core,
    stealer: Stealer<Work>,
    worker: Worker<Work>,
}

impl Cache {
    pub fn new(capacity: usize) -> Result<Self, io::Error> {
        let (worker, stealer) = deque::new();
        let cache = Cache {
            pool: CpuPool::new_num_cpus(),
            core: Core::new()?,
            worker: worker,
            stealer: stealer,
        };

        cache.start(capacity);
        Ok(cache)
    }

    pub fn start(&self, capacity: usize) {
        let stealer = self.stealer.clone();
        let work = future::loop_fn(
            (stealer.clone(), LruCache::new(capacity)),
            |(stealer, mut store): (Stealer<Work>, Store)| {
                match stealer.steal() {
                    Stolen::Empty => (),
                    Stolen::Abort => (),
                    Stolen::Data(work) => {
                        let (snd, msg) = work;
                        let success = match handle(&mut store, msg) {
                            Ok(msg) => snd.send(msg),
                            Err(e) => snd.send(handle_error(&e)),
                        };
                        match success {
                            Ok(_) => (),
                            Err(e) => println!("Failed to send: {}.", e),
                        }
                    }
                };
                future::ok(future::Loop::Continue((stealer, store)))
            },
        );

        self.core.handle().spawn(self.pool.spawn(work));
    }
}

impl Cache {
    pub fn process(&self, message: Message, snd: Sender<Message>) {
        self.worker.push((snd, message));
    }
}

fn handle(store: &mut Store, message: Message) -> Result<Message, error::Error> {
    let op = message.op();
    let (key, payload) = message.consume_request()?;

    let response = match op {
        Op::Set => {
            let key = key;
            let payload = payload.ok_or_else(|| "no payload given to set op")?;
            store.insert(key, payload);
            message::response(Op::Set, Code::Ok, None)
        }

        Op::Get => {
            if let Some(ref mut payload) = store.get_mut(key.as_slice()) {
                message::response(Op::Get, Code::Hit, Some(payload.clone()))
            } else {
                message::response(Op::Get, Code::Miss, None)
            }
        }

        Op::Del => {
            // Probably never going to do this
            message::response(Op::Del, Code::Ok, None)
        }
        Op::Stats => {
            message::response(
                Op::Stats,
                Code::Ok,
                Some(message::payload(store.len() as u32, vec![])),
            )
        }
    };

    Ok(response)
}

fn handle_error(err: &error::Error) -> Message {
    message::response(
        Op::Get,
        Code::Error,
        Some(message::payload(
            0,
            err.description().to_owned().into_bytes(),
        )),
    )
}
