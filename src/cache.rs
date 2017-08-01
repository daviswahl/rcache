use message::{self, Message, Op, Code, Payload};
use tokio_core::reactor::Core;
use std::error::Error;
use futures::sync::oneshot::Sender;
use futures_cpupool::CpuPool;
use futures::future;
use std::io;
use error;
use lru_cache::LruCache;
use deque::{self, Worker, Stealer, Stolen};


type Store = LruCache<Vec<u8>, Payload>;

type Work = (Sender<Message>, Message);

/// A thread safe wrapper around `LruCache` that synchronizes reads/writes via a single
/// threaded worker that reads requests from a dequeue and pushes responses into a channel
/// provided by the request (`Work`) payload.
pub struct Cache {
    pool: CpuPool,
    core: Core,
    stealer: Stealer<Work>,
    worker: Worker<Work>,
}

impl Cache {
    /// Initialize a new `Cache` with `capacity` and start the worker thread.
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

    /// Start the stealer thread, which has unsynchronized access to the underlying store.
    /// `Work` is pushed to the worker via the deque. `Work` is a (Sender<Message>, Message) pair
    /// where `Message` is a request to do work on the store and `Sender` is a channel to send the result.
    ///
    /// TODO: using `loop_fn` doesn't do what I thought, and this thread currently pegs the CPU just waiting for work.
    /// I think I need to make the work queue a pollable stream so that we can wait for new work without pegging the CPU.
    pub fn start(&self, capacity: usize) {
        let stealer = self.stealer.clone();
        // Loop infinitely, attempting to steal work from the deque.
        // When work is obtained, it's dispatched to the `handle` method, which returns a Result containing
        // the `Message::Response` variant. The response will be returned via the `Sender`
        let work = future::loop_fn(
            (stealer, LruCache::new(capacity)),
            |(stealer, mut store): (Stealer<Work>, Store)| {
                match stealer.steal() {
                    Stolen::Empty => (), // Continue
                    Stolen::Abort => (), // TODO: Handle aborts, the obvious manner of doing this doesn't seem to be working
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

    /// Push work onto the queue. `snd` is a `futures::sync::oneshot::Sender<Message>`. When the
    /// worker has completed the request, it will send its `Message::Response` via the sender.
    pub fn process(&self, message: Message, snd: Sender<Message>) {
        self.worker.push((snd, message));
    }
}

/// Handle the request. `Message` is a `Message::Request` variant from the front end.
/// The response message should be a `Message::Response` variant.
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

        // TODO
        Op::Del => {
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

/// Creates a `Message::Response`, setting the error code and
/// and passing the error description as the payload. Responses with an error code should
/// enforce the invariant that the payload contain a UTF8-encoded string, so that clients
/// can safely decode the payload for human consumption.
///
/// TODO: match over the error kind and translate it into an appropriate error for the front end.
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
