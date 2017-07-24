#![feature(test)]
extern crate test;
extern crate rcache;
extern crate tokio_service;
extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
use std::env;
use rcache::client;
use rcache::service;
use rcache::cache;
use rcache::message::{MessageBuilder, Op};
use futures::Future;
use futures_cpupool::CpuPool;
use tokio_core::reactor::Core;
use tokio_service::{Service, NewService};
use std::thread;


fn main() {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "server" => do_server(),
            "client" => {
                let mut children = vec![];
                for i in 0..10 {
                   children.push(thread::spawn(move|| do_client(i)));
                }

                for child in children {
                    child.join().unwrap();
                }
            },
            _ => (),
        }
    }
}

fn do_server() {
    service::serve(
        "127.0.0.1:12345".parse().unwrap(),
        || Ok(service::LogService { inner: service::CacheService { cache: cache::Cache {}} }),

    ).unwrap();
}

fn do_client(i: i64) {
    let mut core = Core::new().unwrap();

    let client = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());

    core.run(client.and_then(|client| {
        let mut message_builder = MessageBuilder::new();
        {
            message_builder
                .set_op(Op::Set)
                .set_type_id(i as u32)
                .set_key("foo".to_owned().into_bytes())
                .set_payload("bar".to_owned().into_bytes());
        }
            client.call(
                message_builder
                    .finish()
                    .unwrap(),
            ).and_then(move|resp|
                client.call(resp)
            )
    })).unwrap();
}
