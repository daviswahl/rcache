#![feature(test)]
extern crate test;
extern crate rcache;
extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
extern crate tokio_service;
extern crate rand;
extern crate time;
extern crate clap;

use rcache::client;
use rcache::service;
use rcache::cache;
use std::error::Error;
use std::net::SocketAddr;
use rcache::message::{Message, Op, Code};
use futures::Future;
use std::sync::Arc;
use tokio_core::reactor::Core;
use rcache::stats::Stats;
use clap::{Arg, App, SubCommand, ArgMatches};


static DEFAULT_CACHE_SIZE: usize = 2000000;

fn main() {
    let set = SubCommand::with_name("SET")
        .arg(Arg::with_name("KEY").required(true).index(1))
        .arg(Arg::with_name("VALUE").required(true).index(2));

    let get = SubCommand::with_name("GET").arg(Arg::with_name("KEY").required(true).index(1));

    let stats = SubCommand::with_name("STATS").about("Retrieves stats from given server");

    let client = SubCommand::with_name("client")
        .about("Run a client command on server at given address")
        .subcommand(get)
        .subcommand(set)
        .subcommand(stats);

    let server = SubCommand::with_name("server")
        .about("Start a server at given address")
        .arg(Arg::with_name("Cache Size").long("cache_size").help(
            "Maximum number of entries in cache, default: 2,000,000",
        ));

    let matches = App::new("rcache")
        .version("0.1")
        .author("Davis Wahl <daviswahl@gmail.com>")
        .about("A PoC memcached implementation")
        .arg(
            Arg::with_name("Socket Address")
                .help(
                    "Address to bind to if running server subcommand, or theaddress \
                of the rcache server if running a client command",
                )
                .required(true)
                .index(1),
        )
        .subcommand(client)
        .subcommand(server)
        .get_matches();

    match run(&matches) {
        Ok(result) => println!("{}", result),
        Err(err) => println!("err: {}", err),
    }
}

fn run(matches: &ArgMatches) -> Result<String, String> {
    // Unwraps in here are safe because clap has already validated that required params are present
    let addr: SocketAddr = matches
        .value_of("Socket Address")
        .unwrap()
        .parse()
        .map_err(|_| "Failed to parse socket address.")?;

    if let Some(matches) = matches.subcommand_matches("server") {
        let cache_size: usize = matches
            .value_of("cache_size")
            .map(|s| s.parse().unwrap_or_else(|_| DEFAULT_CACHE_SIZE))
            .unwrap_or_else(|| DEFAULT_CACHE_SIZE);
        run_server(addr, cache_size).map(|_| "success".to_owned())
    } else if let Some(matches) = matches.subcommand_matches("client") {
        run_client(addr, matches)
    } else {
        Err("Unrecognized sub-command. ".to_owned() + matches.usage())
    }
}

fn run_client(addr: SocketAddr, matches: &ArgMatches) -> Result<String, String> {
    let mut core = Core::new().map_err(|e| e.description().to_owned())?;
    let client = client::Client::connect(&addr, &core.handle());

    // Unwraps in here are safe because clap has already validated that required params are present
    let client_cmd = |client: client::Client| match matches.subcommand() {
        ("GET", Some(matches)) => {
            // handle GET
            let key = matches.value_of("KEY").unwrap();
            client.get(key.to_owned().into_bytes())
        }
        ("SET", Some(matches)) => {
            // handle SET
            let key = matches.value_of("KEY").unwrap();
            let value = matches.value_of("VALUE").unwrap();
            client.set(key.to_owned().into_bytes(), value.to_owned().into_bytes())
        }
        ("STATS", _) => client.stats(),
        _ => unimplemented!(),
    };


    let exec = client.and_then(client_cmd).map(|msg| handle_response(&msg));

    // TODO: Don't want to unwrap here, should propagate error to top level
    core.run(exec).expect("core failure")
}

fn run_server(addr: SocketAddr, cache_size: usize) -> Result<(), String> {
    let cache = cache::Cache::new(cache_size).unwrap();
    cache.start(cache_size);

    // TODO: Figure out the idiomatic way to build up these middleware
    let service = service::StatService {
        stats: Arc::new(Stats::default()),
        inner: service::CacheService { cache: Arc::new(cache) },
    };

    service::serve(addr, service).map_err(|e| e.description().to_owned())
}

// Decode utf-8 strings if the message type_id is 1, otherwise just defer to builtin formatter
fn handle_response(msg: &Message) -> Result<String, String> {
    match (msg.op(), msg.code(), msg.payload()) {
        // Get
        (Op::Get, Code::Hit, Some(payload)) => {
            // Payload is a utf8 encoded string
            if payload.type_id() == 1 {
                String::from_utf8(payload.data().to_owned()).map_err(|_| {
                    "expected a utf8-encoded string".to_owned()
                })
            } else {
                Ok(format!("{}", msg))
            }
        }
        (Op::Stats, _, Some(payload)) => {
            String::from_utf8(payload.data().to_owned()).map_err(|_| {
                "expected a utf8-encoded string".to_owned()
            })
        }
        _ => Ok(format!("{}", msg)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;
    use std::thread;
    use rand::Rng;
    use rcache::message::{self, Op};
    use tokio_service::Service;

    fn build_sets(count: usize) -> Vec<Message> {
        let mut rng = rand::thread_rng();
        (0..count)
            .map(|_| {
                let mut key = [0; 5];
                rng.fill_bytes(&mut key);
                let mut value = [0; 150];
                rng.fill_bytes(&mut value);
                message::request(
                    Op::Set,
                    key.to_vec(),
                    Some(message::payload(2, value.to_vec())),
                )
            })
            .collect()
    }

    fn build_gets(count: usize) -> Vec<Message> {
        let mut rng = rand::thread_rng();

        (0..count)
            .map(|_| {
                let mut key = [0; 5];
                rng.fill_bytes(&mut key);
                message::request(Op::Get, key.to_vec(), None)
            })
            .collect()
    }

    /// TODO: Better benchmarking.
    #[bench]
    fn bench_gets_full_cache(b: &mut Bencher) {
        let mut core = Core::new().unwrap();
        let addr: SocketAddr = "127.0.0.1:12345".parse().unwrap();

        thread::spawn(move || run_server(addr.clone(), 200000));
        let duration = std::time::Duration::new(0, 1000);
        thread::sleep(duration);

        // fill cache
        let sets = build_sets(200000);
        let handle = core.handle();
        core.run(client::Client::connect(&addr, &handle).and_then(
            move |client| {
                let futs = sets.into_iter().map(move |msg| client.call(msg.clone()));
                futures::future::join_all(futs)
            },
        )).unwrap();

        let ops = [&build_gets(25000)[..], &build_sets(25000)[..]].concat();
        b.iter(|| {
            let client = client::Client::connect(&addr, &core.handle());
            let requests = client.and_then(|client| {
                let futs = ops.iter().map(move |msg| client.call(msg.clone()));
                futures::future::join_all(futs)
            });
            core.run(requests).unwrap();

            let stats =
                client::Client::connect(&addr, &core.handle()).and_then(|client| client.stats());
            println!("{}", handle_response(&core.run(stats).unwrap()).unwrap())
        })

    }

}
