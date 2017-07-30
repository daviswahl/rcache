#![feature(test)]
extern crate test;
extern crate rcache;
extern crate tokio_service;
extern crate tokio_core;
extern crate futures;
extern crate futures_cpupool;
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
                    "Address to bind to if running server subcommand, or the address \
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
    let addr: SocketAddr = matches
        .value_of("Socket Address")
        .unwrap() // Safe to unwrap because clap has validated that the addr is present
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
    service::serve(
        addr,
        service::StatService {
            stats: Arc::new(Stats::default()),
            inner: service::CacheService {
                cache: Arc::new(cache::Cache::new(cache_size).unwrap()),
            },
        },
    ).map_err(|e| e.description().to_owned())
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
                format!("expected a utf8-encoded string")
            })
        }
        _ => Ok(format!("{}", msg)),
    }
}
