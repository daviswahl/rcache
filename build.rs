extern crate capnpc;

fn main() {
    ::capnpc::CompilerCommand::new()
        .file("schema/cache.capnp")
        .run()
        .unwrap();
}
