#![feature(conservative_impl_trait)]
#![feature(try_from)]
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

pub mod codec;
pub mod client;
pub mod service;
pub mod message;

pub mod cache_capnp {
    include!(concat!(env!("OUT_DIR"), "/schema/cache_capnp.rs"));
}
