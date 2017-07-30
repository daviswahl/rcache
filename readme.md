# rcache

rcache is a basic, featureless memory cache with a TCP frontend analogous to memcached, written in Rust. It is not 
production ready, and it's unlikely that it ever will be. Nevertheless, it is performant and stable, and is a good example 
of what a naive developer can achieve with Rust. 

This is 100% a learning project. My goals included learning tokio-rs, implementing a binary protocol (my first), and delving 
into lower level/systems programming. 

TODOs include better benchmarks, more featureful clients (for rust and the command line), support for a more complete set of commands, and improving the binary protocol to at least include CRC checks.

## Features

1) Based on [tokio](https://tokio.rs), the asynchronous-io library for Rust.
2) The TCP frontend speaks a multiplexed-binary protocol, detailed (poorly) in [src/codec.rs](src/codec.rs).
3) Currently supports `GET`, `SET`, and `DEL` commands. `CAS` is conspicuously absent, but will be along eventually.
4) Storage is backed by an LRU cached based on a Linked Hash Map (provided by the [lru-cache](https://crates.io/crates/lru-cache) crate), 
all operations are synchronized around a single mutex, which is the obvious performance bottleneck.

