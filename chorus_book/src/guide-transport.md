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
# use chorus_lib::core::{ChoreographyLocation, ChoreoOp, Choreography, ProjectorForAL};
# use chorus_lib::hlist;
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice, Bob);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#     }
# }

// Crate available locations for Projector
type AL = hlist!(Alice, Bob);

let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
let transport = LocalTransport::from(&[Alice::name(), Bob::name()]);
{
    // create a clone for Alice
    let transport = transport.clone();
    handles.push(thread::spawn(move || {
        let p = ProjectorForAL::<AL>::new(Alice, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
{
    // create another for Bob
    let transport = transport.clone();
    handles.push(thread::spawn(move || {
        let p = ProjectorForAL::<AL>::new(Bob, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
```

### The HTTP Transport

The `http` transport is used to execute choreographies on different machines. This is useful for executing choreographies in a distributed system.

To use the `http` transport, import the `HttpTransport` struct from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
use chorus_lib::transport::http::HttpTransport;
```

The `new` constructor takes the name of the projection target and "configuration" of type `std::collections::HashMap<&'static str, (&'static str, u32)>`. The configuration is a map from location names to the hostname and port of the location.

```rust
{{#include ./header.txt}}
# use chorus_lib::transport::http::HttpTransport;
# use std::collections::HashMap;
let mut config = HashMap::new();
config.insert(Alice::name(), ("localhost", 8080));
config.insert(Bob::name(), ("localhost", 8081));
let transport = HttpTransport::new(Alice::name(), &config);
```

In the above example, the transport will start the HTTP server on port 8080 on localhost. If Alice needs to send a message to Bob, it will use `http://localhost:8081` as the destination.

## Creating a Custom Transport

You can also create your own transport by implementing the `Transport` trait. See the API documentation for more details.
