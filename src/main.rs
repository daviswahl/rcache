#![feature(test)]
extern crate test;
extern crate rcache;
extern crate tokio_service;
extern crate tokio_core;
extern crate futures;
use std::env;
use rcache::client;
use rcache::service;
use rcache::message::{Message, Payload, Op};
use futures::Future;
use tokio_core::reactor::Core;
use tokio_service::Service;


fn main() {
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "server" => {
                service::serve(
                    "127.0.0.1:12345".parse().unwrap(),
                    service::LogService { inner: service::CacheService },
                )
            }
            "client" => do_client(),
            _ => (),
        }
    }
}

fn do_client() {
    let mut core = Core::new().unwrap();

    let client = client::Client::connect(&"127.0.0.1:12345".parse().unwrap(), &core.handle());
    let message = Message {
        op: Op::Set,
        key: "1234".to_owned().into_bytes(),
        payload: Payload {
            type_id: 123,
            data: "1232350458019238".to_owned().into_bytes(),
        },
    };

    core.run(client.and_then(|client| {
        let futs = (0..10).map(move |_| client.call(message.clone()));
        futures::future::join_all(futs)
    })).unwrap();
}
#[cfg(test)]
mod tests {

    use super::*;
    use test::Bencher;
    #[bench]
    fn bench1(b: &mut Bencher) {}
}
