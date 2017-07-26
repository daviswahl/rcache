#![feature(conservative_impl_trait)]
#![feature(try_from)]
#![feature(plugin)]
#![plugin(clippy)]

#![allow(match_same_arms)]
extern crate capnp;
extern crate time;
extern crate capnp_futures;
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

pub mod codec;
pub mod client;
pub mod service;
pub mod message;
pub mod proto;
pub mod cache;
pub mod stats;
pub mod error;
