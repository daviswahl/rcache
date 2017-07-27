use message::{self, Message, Op, Code, Payload};
use tokio_core::reactor::Core;
use std::error::Error;
use futures::sync::oneshot::Sender;
use futures_cpupool::CpuPool;
use std::sync::{Arc, Mutex};
use futures::future;
use std::io;
use error;
use lru_cache::LruCache;


type Store = Arc<Mutex<LruCache<Vec<u8>, Payload>>>;

/// `Cache`
pub struct Cache {
    pool: CpuPool,
    core: Core,
    store: Store,
}

impl Cache {
    pub fn new(capacity: usize) -> Result<Self, io::Error> {
        Ok(Cache {
            pool: CpuPool::new_num_cpus(),
            core: Core::new()?,
            store: Arc::new(Mutex::new(LruCache::new(capacity))),
        })
    }
}

impl Cache {
    pub fn process(&self, message: Message, snd: Sender<Message>) {
        let store = self.store.clone();
        let work = || {
            let response = match handle(store, message) {
                Ok(msg) => msg,
                Err(err) => handle_error(&err),
            };

            match snd.send(response) {
                Ok(_) => future::ok(()),
                Err(e) => {
                    println!("failed to send message: {:?}", e);
                    future::ok(())
                }
            }
        };

        self.core.handle().spawn(self.pool.spawn_fn(work))
    }
}

fn handle(store: Store, message: Message) -> Result<Message, error::Error> {
    let op = message.op();
    let (key, payload) = message.consume_request()?;

    let response = match op {
        Op::Set => {
            let key = key;
            let payload = payload.ok_or_else(|| "no payload given to set op")?;

            store
                .lock()
                .map(|mut store| { store.insert(key, payload); })
                .map_err(|e| {
                    error::Error::new(error::ErrorKind::Other, e.description())
                })?;

            message::response(Op::Set, Code::Ok, None)
        }

        Op::Get => {
            store
                .lock()
                .map(|mut store| if let Some(ref mut payload) =
                    store.get_mut(key.as_slice())
                {
                    message::response(Op::Get, Code::Ok, Some(payload.clone()))
                } else {
                    message::response(Op::Get, Code::Miss, None)
                })
                .map_err(|e| {
                    error::Error::new(error::ErrorKind::Other, e.description())
                })?
        }

        Op::Del => {
            // Probably never going to do this
            message::response(Op::Del, Code::Ok, None)
        }
        Op::Stats => {
            store
                .lock()
                .map(|store| {
                    message::response(Op::Stats, Code::Ok, Some(message::payload(store.len() as u32, vec![])))
                })
                .map_err(|e| {
                    error::Error::new(error::ErrorKind::Other, e.description())
                })?
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
