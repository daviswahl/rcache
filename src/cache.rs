use message::{Message, MessageBuilder, Op, Code};
use tokio_core::reactor::{Core, Remote};
use std::error::Error;
use futures::sync::oneshot::Sender;
use futures_cpupool::CpuPool;
use std::thread;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use futures::{Future, future, BoxFuture};
use rand::{self, Rng};
use std::time::Duration;
use std::io;

/// `Cache`
pub struct Cache {
    pool: CpuPool,
    core: Core,
    data: Arc<RwLock<HashMap<Vec<u8>, (u32, Vec<u8>)>>>,
}

impl Cache {
    pub fn new() -> Result<Self, io::Error> {
        Ok(Cache {
            pool: CpuPool::new_num_cpus(),
            core: Core::new()?,
            data: Arc::new(RwLock::new(HashMap::new())),
        })
    }
}

impl Cache {
    pub fn process(&self, message: Message, snd: Sender<Message>)  {
        let data = self.data.clone();
        let work = move || match message.op() {
            Op::Set => {
                let (key, payload) = message.consume();

                data.write().map(|mut cache| {
                    cache.insert(key.unwrap(), payload.unwrap().into());
                }).unwrap();

                future::ok(snd.send(MessageBuilder::default().set_op(Op::Set).set_code(Code::Ok).response().unwrap()).unwrap())
            }

            Op::Get => {
                let key = message.key().unwrap();
                data.read()
                    .map(|cache| if let Some(&(ref type_id, ref data)) =
                        cache.get(key)
                    {
                        let mut mb = MessageBuilder::new();
                        {
                            mb.set_type_id(*type_id).set_payload(data.clone()).set_op(Op::Get).set_code(Code::Ok);
                        }
                        snd.send(mb.response().unwrap());
                    } else {
                        let mut mb = MessageBuilder::default();
                        {
                            mb.set_op(Op::Get).set_code(Code::Miss);
                        }
                        snd.send(mb.response().unwrap());
                    })
                    .unwrap();
                future::ok(())

            }

            Op::Del => {
                // Probably never going to do this
                let mut mb = MessageBuilder::new();
                {
                    mb.set_op(Op::Del).set_code(Code::Ok);
                }
                snd.send(mb.into_response().unwrap());
                future::ok(())
            }
            Op::Stats => {
                snd.send(message);
                future::ok(())
            }
        };

        self.core.handle().spawn(self.pool.spawn_fn(work))
    }
}
