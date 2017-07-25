#![feature(test)]
extern crate test;
extern crate rcache;
extern crate tokio_service;
extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
extern crate rand;
use std::env;
use rcache::client;
use rcache::service;
use rcache::cache;
use rcache::message::{MessageBuilder, Op};
use futures::Future;
use std::sync::Arc;
use futures_cpupool::CpuPool;
use tokio_core::reactor::Core;
use tokio_service::{Service, NewService};
use std::thread;
use rand::Rng;


fn main() {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "server" => do_server(),
            "client" => {
                let mut i = 0;
                let mut children = vec![];
                loop {
                    children.push(thread::spawn(move || { do_client(i); }));
                    i = i + 1;

                    if children.len() > 20 {
                        while let Some(child) = children.pop() {
                            child.join();
                        }
                    }
                    println!("{}", i);
                }
            }
            _ => (),
        }
    }
}

fn do_server() {
    service::serve("127.0.0.1:12345".parse().unwrap(), || {
        Ok(service::LogService {
            inner: service::CacheService { cache: cache::Cache::new().unwrap() },
        })
    }).unwrap();
}
fn do_client(i: u32) {
    let mut core = Core::new().unwrap();

    let client = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());

    let mut rng = rand::thread_rng();
    let op = rng.choose(&[Op::Set, Op::Get, Op::Del]).map(|&x| x);

    let key = rng.choose(
        &[
            "foo",
            "bar",
            "batz",
            "zig",
            "zag",
            "zowie",
            "wowie",
            "blammo",
            "rammo",
        ],
    ).map(|&x| x);
    let mut buf: [u8; 100] = [0; 100];

    rng.fill_bytes(&mut buf);

    core.run(client.and_then(move |client| {
        let mut message_builder = MessageBuilder::new();
        {
            message_builder
                .set_op(op.unwrap())
                .set_type_id(i as u32)
                .set_key(key.unwrap().to_owned().into_bytes())
                .set_payload(buf.to_owned());
        }
        client.call(message_builder.finish().unwrap())
    })).unwrap();
}
