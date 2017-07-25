#![feature(test)]
extern crate test;
extern crate rcache;
extern crate tokio_service;
extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
extern crate rand;
extern crate time;
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
use rcache::stats::Stats;


fn main() {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "server" => do_server(),
            "client3" => do_client2(),
            "client2" => loop { do_client(1) }
            "client" => {
                let start = time::now();
                let mut children = vec![];
                for i in 0..100 {
                    children.push(thread::spawn(move || { do_client(i); }));
                }
                while let Some(child) = children.pop() {
                    child.join();
                }

                let delta = time::now() - start;
                println!("delta: {}", delta.num_milliseconds());
            }
            "stats" => {
                let mut core = Core::new().unwrap();
                let client =
                    client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());
                let mut msg = MessageBuilder::new().set_op(Op::Stats).set_key("foo".into()).request().unwrap();
                let req = client.and_then(|client| {
                    client.call(msg).and_then(|resp| {
                        println!(
                            "{}",
                            String::from_utf8(resp.payload().unwrap().data().to_owned()).unwrap()
                        );
                        Ok(())
                    })
                });
                core.run(req);
            }
            _ => (),
        }
    }
}

fn do_server() {
    service::serve(
        "127.0.0.1:12345".parse().unwrap(),
        service::StatService {
            stats: Arc::new(Stats::new()),
            inner: {
                service::LogService {
                    inner: service::CacheService { cache: Arc::new(cache::Cache::new().unwrap()) },
                }
            },
        },
    ).unwrap();
}

fn do_client2() {
    let mut message_builder = MessageBuilder::new();
    {
        message_builder
            .set_op(Op::Set)
            .set_type_id(1)
            .set_key("foo".to_owned().into_bytes())
            .set_payload("bar".to_owned().into_bytes());
    }
    let msg1 = message_builder.request().unwrap();

    {
        message_builder.set_op(Op::Get);
    }

    let msg2 = message_builder.request().unwrap();

    {
        let mut core = Core::new().unwrap();
        let client = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());
        let client2 = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());

        core.run(client.and_then(|client| {
            client.call(msg1).map(|resp| println!("resp1: {:?}", resp))
        })).unwrap();
        core.run(client2.and_then(|client| {
            client.call(msg2).map(|resp| println!("resp2: {:?}", resp))
        })).unwrap();
    }
}
fn do_client(i: u32) {
    let mut core = Core::new().unwrap();

    let client = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());

    let mut messages = vec![];

    for i in 0..500 {
        let mut rng = rand::thread_rng();
        let op = rng.choose(&[Op::Get, Op::Del, Op::Set]).map(|&x| x);

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
        let mut message_builder = MessageBuilder::new();
        {
            message_builder
                .set_op(op.unwrap())
                .set_type_id(i)
                .set_key(key.unwrap().to_owned().into_bytes())
                .set_payload(buf.to_owned());
        }
        messages.push(message_builder.request().unwrap())
    }

    core.run(client.and_then(move |client| {
        let mut futs = vec![];
        for msg in messages {
            futs.push(client.call(msg));
        }
        futures::future::join_all(futs)
    })).unwrap();
}
