use message::{Message, MessageBuilder, Op, Code};
use tokio_core::reactor::Core;
use std::error::Error;
use futures::sync::oneshot::Sender;
use futures_cpupool::CpuPool;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use futures::future;
use std::io;
use error;
use lru_cache::LruCache;

/// `Cache`
pub struct Cache {
    pool: CpuPool,
    core: Core,
    store: Store,
}

type Store = Arc<RwLock<LruCache<Vec<u8>, (u32, Vec<u8>)>>>;
impl Cache {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Cache {
            pool: CpuPool::new_num_cpus(),
            core: Core::new()?,
            store: Arc::new(RwLock::new(LruCache::new(40000))),
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
    let (key, payload) = message.consume();
    let mut builder = MessageBuilder::default();
    {
        builder.set_op(op);
    }

    match op {
        Op::Set => {
            let key = key.ok_or_else(|| "no key given to set op")?;
            let payload = payload.ok_or_else(|| "no payload given to set op")?;

            store
                .write()
                .map(|mut store| { store.insert(key, payload.into()); })
                .map_err(|e| {
                    error::Error::new(error::ErrorKind::Other, e.description())
                })?;

            builder.set_code(Code::Ok);
        }

        Op::Get => {
            let key = key.ok_or_else(|| "no key given to get op")?;
            store
                .write()
                .map(|mut store| if let Some(&mut (ref type_id, ref data)) =
                    store.get_mut(key.as_slice())
                {
                    builder
                        .set_type_id(*type_id)
                        .set_payload(data.clone())
                        .set_code(Code::Ok);
                } else {
                    builder
                        .set_op(Op::Get)
                        .set_code(Code::Miss);
                })
                .map_err(|e| {
                    error::Error::new(error::ErrorKind::Other, e.description())
                })?
        }

        Op::Del => {
            // Probably never going to do this
            builder.set_code(Code::Ok);
        }
        Op::Stats => {
            builder.set_code(Code::Ok);
        }
    }
    builder.into_response()
}

fn handle_error(err: &error::Error) -> Message {
    MessageBuilder::default()
        .set_op(Op::Get)
        .set_code(Code::Error)
        .set_payload(err.description().to_owned().into_bytes())
        .response()
        .unwrap()
}
