@0x970aa9edf26e100e;

enum Type {
    foo @0;
}
enum Code {
    success @0;
    failure @1;
}

enum Op {
    get @0;
    set @1;
    del @2;
}

struct Request(Value) {
    op @0 :Op;
    key @1 :Data;
    payload @2 :Payload(Value);
}

struct Response(Value) {
    requestId @0 :Text;
    code       @1 :Code;
    payload    @2 :Payload(Value);
}

struct Payload(Value){
    type @0 :Type;
    data @1 :Value;
}

struct Foo {
    name @0 :Text;
}
