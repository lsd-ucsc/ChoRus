# Transport

In order to execute choreographies, we need to be able to send messages between locations. ChoRus provides a trait `Transport` that abstracts over message transports.

## Built-in Transports

ChoRus provides two built-in transports: `local` and `http`.

### The Local Transport

The `local` transport is used to execute choreographies on the same machine on different threads. This is useful for testing and prototyping. Each `local` transport is defined over `LocalTransportChannel`, which contains the set of `ChoreographyLocation` that the `local` transport operates on. You can build a `LocalTransportChannel` by importing the `LocalTransportChannel` stsruct from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
# use std::sync::Arc;
# use chorus_lib::core::{ChoreographyLocation};
# use chorus_lib::{LocationSet};
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
use chorus_lib::transport::local::LocalTransportChannel;

let transport_channel = Arc::new(LocalTransportChannel::<LocationSet!(Alice, Bob)>::new());
```

To use the `local` transport, first import the `LocalTransport` struct from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
# use std::sync::Arc;
# use chorus_lib::core::{ChoreographyLocation};
# use chorus_lib::{LocationSet};
# #[derive(ChoreographyLocation)]
# struct Alice;
# let transport_channel = Arc::new(LocalTransportChannel::<LocationSet!(Alice)>::new());
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};

let alice_transport = LocalTransport::new(Alice, Arc::clone(&transport_channel));
```

 Then build the transport by using the `LocalTransport::new` associated function, which takes a target location (explained in the [Projector section](./guide-projector.md)) and the `LocalTransportChannel`.

You can construct a `LocalTransport` instance by passing a target location(lained in the [Projector section](./guide-projector.md)) and a `std::sync::Arc` of `LocalTransportChannel` to the `new` method.

Because of the nature of the `Local` transport, you must make a `std::sync::Arc` from the `LocalTransportChannel`, by using the `std::sync::Arc::clone(&local_transport)`.

```rust
# extern crate chorus_lib;
# use std::sync::Arc;
# use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};
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
let transport_channel = Arc::new(LocalTransportChannel::<LocationSet!(Alice, Bob)>::new());
let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
{
    // create a transport for Alice
    let transport = LocalTransport::new(Alice, Arc::clone(&transport_channel));
    handles.push(thread::spawn(move || {
        let p = Projector::new(Alice, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
{
    // create another for Bob
    let transport = LocalTransport::new(Bob, Arc::clone(&transport_channel));
    handles.push(thread::spawn(move || {
        let p = Projector::new(Bob, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
```

### The HTTP Transport

The `http` transport is used to execute choreographies on different machines. This is useful for executing choreographies in a distributed system.

To use the `http` transport, import the `HttpTransport` struct and the `transport_config` macro from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
use chorus_lib::transport::http::HttpTransport;
use chorus_lib::transport_config;
```

The new constructor takes a "configuration" of type TransportConfig. To build the TransportConfig, you should use the macro transport_config and give it a key => value, followed by a comma (if you have more than one key/value), and a comma-separated list of key: values where the key => value is the target ChoreographyLocation and the value is the required information for the target, and each following key: value is a ChoreographyLocation and the required information for it (`(host_name, port)` in this case). For HttpTransport You can think of TransportConfig as a map from locations to the hostname and port of the location. But for a generic Transport, it can contain any kind of information.

```rust
{{#include ./header.txt}}
# use chorus_lib::transport::http::{HttpTransport};
# use chorus_lib::transport_config;
let config = transport_config!(
                Alice => ("0.0.0.0".to_string(), 9010),
                Bob: ("localhost".to_string(), 9011)
            );

let transport = HttpTransport::new(&config);
```

In the above example, the transport will start the HTTP server on port 8080 on localhost. If Alice needs to send a message to Bob, it will use `http://localhost:8081` as the destination.

## Creating a Custom Transport

You can also create your own transport by implementing the `Transport` trait. See the API documentation for more details.


### Note on the location set of the Choreography

Note that when calling `epp_and_run` on a `Projector`, you will get a compile error if the location set of the `Choreography` is not a subset of the location set of the `Transport`. In other words, the `Transport` should have information about every `ChoreographyLocation`  that `Choreography` can talk about. So this will fail:

```rust, compile_fail
# extern crate chorus_lib;
# use std::sync::Arc;
# use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};
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

let transport_channel = Arc::new(LocalTransportChannel::<LocationSet!(Alice)>::new());
let transport = LocalTransport::new(Alice, Arc::clone(&transport_channel));
let projector = Projector::new(Alice, transport);
projector.epp_and_run(HelloWorldChoreography);
```
