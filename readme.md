# rcache

rcache is a basic, featureless memory cache with a TCP frontend analogous to memcached, written in Rust. It is not 
production ready, and it's unlikely that it ever will be. Nevertheless, it is performant and stable, and is a good example 
of what a naive developer can achieve with Rust. 

TODOs include better benchmarks, more featureful clients (for rust and the command line), support for a more complete set of commands, and improving the binary protocol to at least include CRC checks.

## Features

1) Based on [tokio](https://tokio.rs), the asynchronous-io library for Rust.
2) The TCP frontend speaks a multiplexed-binary protocol, detailed (poorly) in [src/codec.rs](src/codec.rs).
3) Currently supports `GET`, `SET`, and `DEL` commands. `CAS` is conspicuously absent, but will be along eventually.
4) Storage is backed by an LRU cached based on a Linked Hash Map (provided by the [lru-cache](https://crates.io/crates/lru-cache) crate), 
all operations are synchronized around a single mutex, which is the obvious performance bottleneck.

## Usage

1) You'll need the nightly version of rust, follow instructions at https://www.rustup.rs/
2) `cargo install`
3) Start a server: `rcache 127.0.0.1:12345 server`
4) Set a key: `rcache 127.0.0.1:12345 client SET foo bar`
5) Get a key: `rcache 127.0.0.1:12345 client GET foo`
6) Get stats: `rcache 127.0.0.1:12345 client STATS`

## Performance

I'm currently working on providing realistic benchmarks. Naive benchmarks show that rcache can handle around 50k/req/s with 
2,000,000 keys in the cache. This benchmarking was done with 100 concurrent clients making 500 requests each. However, I've no doubt that
there were numerous issues in my benchmarking methodology. Even so, it's neat that a weekend implementation project
can get into the same ballpark as memcached.
