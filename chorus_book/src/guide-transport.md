# Transport

In order to execute choreographies, we need to be able to send messages between locations. ChoRus provides a trait `Transport` that abstracts over message transports.

## Built-in Transports

ChoRus provides two built-in transports: `local` and `http`.

### The Local Transport

The `local` transport is used to execute choreographies on the same machine on different threads. This is useful for testing and prototyping.

To use the `local` transport, import the `LocalTransport` struct from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
use chorus_lib::transport::local::LocalTransport;
```

You can construct a `LocalTransport` instance by passing a slice of locations to the `from` method.

Because of the nature of the `Local` transport, you must use the same `LocalTransport` instance for all locations. You can `clone` the `LocalTransport` instance and pass it to the threads.

```rust
# extern crate chorus_lib;
# use chorus_lib::transport::local::LocalTransport;
# use std::thread;
# use chorus_lib::core::{ChoreographyLocation, ChoreoOp, Choreography, Projector};
# use chorus_lib::{LocationSet};
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = LocationSet!(Alice, Bob);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#     }
# }


let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
let transport = LocalTransport::<LocationSet!(Alice, Bob)>::new();
{
    // create a clone for Alice
    let transport = transport.clone();
    handles.push(thread::spawn(move || {
        let p = Projector::new(Alice, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
{
    // create another for Bob
    let transport = transport.clone();
    handles.push(thread::spawn(move || {
        let p = Projector::new(Bob, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
```

### The HTTP Transport

The `http` transport is used to execute choreographies on different machines. This is useful for executing choreographies in a distributed system.

To use the `http` transport, import the `HttpTransport` struct and the `http_config` macro from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
use chorus_lib::transport::http::HttpTransport;
use chorus_lib::http_config;
```

The `new` constructor takes the name of the projection target and "configuration" of type `HttpConfig`. To build the `HttpConfig`, you should use the macro `http_config` and give it a comma separatedlist of key: values where each key is a `ChoreographyLocation` and each value is a tuple of (host_name, port). You can think of configuration as a map from locations to the hostname and port of the location.

```rust
{{#include ./header.txt}}
# use chorus_lib::transport::http::{HttpTransport};
# use chorus_lib::http_config;
# use std::collections::HashMap;

let config = http_config!(Alice: ("localhost", 8080), Bob: ("localhost", 8081));
let transport = HttpTransport::new(Alice, &config);
```

In the above example, the transport will start the HTTP server on port 8080 on localhost. If Alice needs to send a message to Bob, it will use `http://localhost:8081` as the destination.

## Creating a Custom Transport

You can also create your own transport by implementing the `Transport` trait. See the API documentation for more details.


### Note on the location set of the Choreography

Note that when calling `epp_and_run` on a `Projector`, you will get a compile error if the location set of the `Choreography` is not a subset of the location set of the `Transport`. In other words, the `Transport` should have information about every `ChoreographyLocation`  that `Choreography` can talk about. So this will fail:

```rust, compile_fail
# extern crate chorus_lib;
# use chorus_lib::transport::local::LocalTransport;
# use chorus_lib::core::{ChoreographyLocation, Projector, Choreography, ChoreoOp};
# use chorus_lib::{LocationSet};

# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
struct HelloWorldChoreography;
impl Choreography for HelloWorldChoreography {
     type L = LocationSet!(Alice, Bob);
     fn run(self, op: &impl ChoreoOp<Self::L>) {
     }
}

let transport = LocalTransport::<LocationSet!(Alice)>::new();
let projector = Projector::new(Alice, transport);
projector.epp_and_run(HelloWorldChoreography);
```
