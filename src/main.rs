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
use rcache::message::{request, payload, Op};
use futures::Future;
use std::sync::Arc;
use tokio_core::reactor::Core;
use tokio_service::Service;
use std::thread;
use rand::Rng;
use rcache::stats::Stats;


fn main() {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "server" => do_server(),
            "client2" => do_client(),
            "client" => {
                let start = time::now();
                let mut children = vec![];
                for _ in 0..100 {
                    children.push(thread::spawn(move || { do_client(); }));
                }
                while let Some(child) = children.pop() {
                    child.join().unwrap();
                }

                let delta = time::now() - start;
                println!("delta: {}", delta.num_milliseconds());
            }
            "stats" => {
                let mut core = Core::new().unwrap();
                let client =
                    client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());
                let msg = request(Op::Stats, "foo".into(), None);
                let req = client.and_then(|client| {
                    client.call(msg).and_then(|resp| {
                        println!(
                            "{}",
                            String::from_utf8(resp.payload().unwrap().data().to_owned()).unwrap()
                        );
                        Ok(())
                    })
                });
                core.run(req).unwrap();
            }
            _ => (),
        }
    }
}

fn do_server() {
    service::serve(
        "127.0.0.1:12345".parse().unwrap(),
        service::StatService {
            stats: Arc::new(Stats::default()),
            inner: {
                service::LogService {
                    inner: service::CacheService {
                        cache: Arc::new(cache::Cache::new(2000000).unwrap()),
                    },
                }
            },
        },
    ).unwrap();
}

fn do_client() {
    let mut core = Core::new().unwrap();

    let client = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());

    let mut messages = vec![];

    for i in 0..500 {
        let mut rng = rand::thread_rng();
        let op = rng.choose(&[Op::Get, Op::Del, Op::Set]).cloned();

        let mut key_buf: [u8; 4] = [0; 4];
        rng.fill_bytes(&mut key_buf);

        let mut buf: [u8; 100] = [0; 100];
        rng.fill_bytes(&mut buf);
        let msg = request(
            op.unwrap(),
            key_buf.to_vec(),
            Some(payload(i, buf.to_owned())),
        );
        messages.push(msg)
    }

    core.run(client.and_then(move |client| {
        let mut futs = vec![];
        for msg in messages {
            futs.push(client.call(msg));
        }
        futures::future::join_all(futs)
    })).unwrap();
}
