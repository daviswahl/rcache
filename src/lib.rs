#![feature(conservative_impl_trait)]
#![feature(plugin)]
#![plugin(clippy)]

extern crate capnp;
extern crate capnp_futures;
extern crate futures;
extern crate mio_uds;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate deque;
extern crate bytes;

use std::collections::HashMap;
use std::sync::RwLock;
use std::sync::Arc;

pub mod error;

pub mod codec;
pub mod client;
pub mod cache_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/cache_capnp.rs"));
}

pub mod message;
pub mod service;

pub type Cache = Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>;

pub fn new_cache() -> Cache {
    Arc::new(RwLock::new(HashMap::new()))
}


pub fn build_messages(
    mut message: cache_capnp::request::Builder<capnp::any_pointer::Owned>,
    op: cache_capnp::Op,
    key: &str,
    data: Vec<u8>,
) {
    message.set_op(op);
    message.set_key(key.as_bytes());
    wrap_result(data, message);
}

pub fn print_message(reader: capnp::message::Reader<capnp_futures::serialize::OwnedSegments>) {
    let message = reader
        .get_root::<cache_capnp::request::Reader<capnp::any_pointer::Owned>>()
        .unwrap();
    let op = message.get_op().unwrap();
    let key = message.get_key().unwrap();
    let value = message.get_payload().unwrap();
    let data = value.get_data().unwrap();
    let env = data.get_as::<cache_capnp::payload::Reader<capnp::any_pointer::Owned>>()
        .unwrap();
    let tpe = env.get_type().unwrap();
    let data = env.get_data().unwrap();
    use message::FromProto;
    let foo = message::Foo::from_proto(&data.get_as().unwrap()).unwrap();
    match tpe {
        cache_capnp::Type::Foo => println!("is foo"),
    }

    println!("foo: {:?}", foo);
}

pub fn read_value(cache: Cache, key: &[u8]) -> Option<Vec<u8>> {
    let cache = cache.read().unwrap();
    cache.get(key).map(|e| e.clone())
}

pub fn set_value(
    cache: Cache,
    key: &[u8],
    value: cache_capnp::payload::Reader<capnp::any_pointer::Owned>,
) {
    let mut cache = cache.write().unwrap();
    use std::io::BufRead;
    let mut buf: Vec<u8> = vec![];
    let mut data = value.get_data().unwrap();

    let mut builder = capnp::message::Builder::new_default();
    {
        let mut message =
            builder.init_root::<cache_capnp::payload::Builder<capnp::any_pointer::Owned>>();
        message.set_data(data);
    }

    capnp::serialize_packed::write_message(&mut buf, &builder);
    cache.insert(Vec::from(key), buf);
}

pub fn read_message(
    cache: Cache,
    reader: cache_capnp::request::Reader<capnp::any_pointer::Owned>,
) -> Vec<u8> {
    match reader.get_op() {
        Ok(op) => {
            match op {
                cache_capnp::Op::Get => {
                    println!("get!");
                    read_value(cache, reader.get_key().expect("get key")).unwrap_or(vec![])
                }
                cache_capnp::Op::Set => {
                    println!("SET!");
                    let value = reader.get_payload().expect("a value");
                    let key = reader.get_key().expect("A key");
                    set_value(cache, key, value);
                    vec![]
                }
                cache_capnp::Op::Del => vec![],
            }
        }
        Err(e) => {
            println!("Error: {}", e);
            vec![]
        }
    }
}

pub fn wrap_result(
    data: Vec<u8>,
    mut builder: cache_capnp::request::Builder<capnp::any_pointer::Owned>,
) {
    if !data.is_empty() {
        let msg = capnp::serialize_packed::read_message(
            &mut data.as_ref(),
            capnp::message::ReaderOptions::default(),
        );
        builder.set_payload(msg.unwrap().get_root().unwrap()).unwrap();
    }
}
