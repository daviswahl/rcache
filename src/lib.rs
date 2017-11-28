#![feature(conservative_impl_trait)]
#![feature(try_from)]
#![feature(plugin)]
#![feature(test)]
//! # rcache
//! NOTE: If you have a good use for the name `rcache` and want ownership, contat me via github.
//!
//! `rcache` is a basic, featureless memory cache with a TCP frontend analogous to memcached.
//! It is not production ready, and it's unlikely that it ever will be. Nevertheless,
//! it is relatively performant and stable, and is a good example of what a naive developer can
//! achieve with Rust.
//!
//! TODOs include better benchmarks, more featureful clients (for rust and the command line), support
//! for a more complete set of commands, and improving the binary protocol to at least include CRC checks.
//!
//! ## Features
//!
//! - Based on `tokio`
//! - The TCP frontend speaks a multiplexed-binary protocol, detailed (poorly) in src/codec.rs.
//! - Currently supports GET, SET, and DEL commands. CAS is conspicuously absent, but will be along eventually.
//! - Storage is backed by an LRU cached based on a Linked Hash Map (provided by the lru-cache crate),
//! all operations are threaded through a single worker, which has unsynchronized access to the store.
//!
//! ## Usage
//!
//! Start a server: `cargo run -- 127.0.0.1:12345 server`
//!
//! Set a key: `cargo run -- 127.0.0.1:12345 client SET foo bar`
//!
//! Get a key: `cargo run -- 127.0.0.1:12345 client GET foo`
//!
//! Get stats: `cargo run -- 127.0.0.1:12345 client STATS`
//!
//!
//! ## Performance
//!
//! I'm currently working on providing realistic benchmarks. Naive benchmarks show that rcache can
//! handle around 50k/req/s with 2,000,000 keys in the cache. This benchmarking was done with 100 concurrent
//! clients making 500 requests each. However, I've no doubt that there were numerous issues in my
//! benchmarking methodology. Even so, it's neat that a weekend implementation project can get into
//! the same ballpark as memcached.

extern crate time;
extern crate futures;
extern crate futures_cpupool;
extern crate mio_uds;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate deque;
extern crate bytes;
extern crate rand;
extern crate lru_cache;
extern crate test;

pub mod client;
pub mod message;
pub mod cache;
pub mod stats;
pub mod service;

mod codec;
mod proto;
mod error;
