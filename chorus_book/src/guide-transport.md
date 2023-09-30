# Transport

In order to execute choreographies, we need to be able to send messages between locations. ChoRus provides a trait `Transport` that abstracts over message transports.

## Built-in Transports

ChoRus provides two built-in transports: `local` and `http`.

### The Local Transport

The `local` transport is used to execute choreographies on the same machine on different threads. This is useful for testing and prototyping. Each `local` transport is defined over `LocalTransportChannel`, which contains the set of `ChoreographyLocation` that the `local` transport operates on. You can build a `LocalTransportChannel` by importing the `LocalTransportChannel` struct from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
# use chorus_lib::core::{ChoreographyLocation, LocationSet};
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
use chorus_lib::transport::local::LocalTransportChannel;

let transport_channel = LocalTransportChannel::new().with(Alice).with(Bob);
```

To use the `local` transport, first import the `LocalTransport` struct from the `chorus_lib` crate.

Then build the transport by using the `LocalTransport::new` associated function, which takes a target location (explained in the [Projector section](./guide-projector.md)) and the `LocalTransportChannel`.

```rust
# extern crate chorus_lib;
# use chorus_lib::core::{ChoreographyLocation, LocationSet};
# #[derive(ChoreographyLocation)]
# struct Alice;
# use chorus_lib::transport::local::LocalTransportChannel;
# let transport_channel = LocalTransportChannel::new().with(Alice);
use chorus_lib::transport::local::{LocalTransport};

let alice_transport = LocalTransport::new(Alice, transport_channel.clone());
```

Because of the nature of the `Local` transport, you must use the same `LocalTransportChannel` instance for all locations. You can `clone` the `LocalTransprotChannel` instance and pass it to each `Projector::new` constructor.

```rust
# extern crate chorus_lib;
# use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};
# use std::thread;
# use chorus_lib::core::{ChoreographyLocation, ChoreoOp, Choreography, Projector, LocationSet};
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
let transport_channel = LocalTransportChannel::new().with(Alice).with(Bob);
let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
{
    // create a transport for Alice
    let transport = LocalTransport::new(Alice, transport_channel.clone());
    handles.push(thread::spawn(move || {
        let p = Projector::new(Alice, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
{
    // create another for Bob
    let transport = LocalTransport::new(Bob, transport_channel.clone());
    handles.push(thread::spawn(move || {
        let p = Projector::new(Bob, transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
}
```

### The HTTP Transport

The `http` transport is used to execute choreographies on different machines. This is useful for executing choreographies in a distributed system.

To use the `http` transport, import the `HttpTransport` struct and the `HttpTransportConfig` type alias from the `chorus_lib` crate.

```rust
# extern crate chorus_lib;
use chorus_lib::transport::http::{HttpTransport, HttpTransportConfig};
```

The primary constructor requires an argument of type `HttpTransportConfig`. To create an instance of this configuration, start with `HttpTransportConfig::for_target(target_location, (hostname, port))`. It will create set a projection target and the hostname and port to listen on. Then, provide information to connect to other locations by method-chaining the `.with(other_location, (hostname, port))` method. You can think of `HttpTransportConfig` as a mapping from locations to their hostnames and ports.

```rust
{{#include ./header.txt}}
# use chorus_lib::transport::http::{HttpTransport, HttpTransportConfig};
let config = HttpTransportConfig::for_target(Alice, ("localhost".to_string(), 8080))
                .with(Bob, ("localhost".to_string(), 8081));

let transport = HttpTransport::new(config);
```

In the above example, the transport will start the HTTP server on port 8080 on localhost. If Alice needs to send a message to Bob, it will use `http://localhost:8081` as the destination.

## Creating a Custom Transport

You can also create your own transport by implementing the `Transport` trait. It might be helpful first build a `TransportConfig` to have the the information that you need for each `ChoreographyLocation`, and then have a constructor that takes the `TransportConfig` and builds the `Transport` based on it. While the syntax is similar to `HttpTransportConfig`, the type of information for each `ChoreographyLocation` might diverge from the `(host_name, port)` format presented here. In some cases, the `target_information` could even have a different type than the following `other_location_information` types. But all the `other_location_information`s should have the same type.

```rust
{{#include ./header.txt}}
# use chorus_lib::transport::TransportConfig;
let config = TransportConfig::for_target(Alice, ())
                .with(Bob, ("localhost".to_string(), 8081))
                .with(Carol, ("localhost".to_string(), 8082));
```

See the API documentation for more details.

### Note on the location set of the Choreography

Note that when calling `epp_and_run` on a `Projector`, you will get a compile error if the location set of the `Choreography` is not a subset of the location set of the `Transport`. In other words, the `Transport` should have information about every `ChoreographyLocation` that `Choreography` can talk about. So this will fail:

```rust, compile_fail
# extern crate chorus_lib;
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

let transport_channel = LocalTransportChannel::<LocationSet!(Alice)>::new();
let transport = LocalTransport::new(Alice, transport_channel.clone());
let projector = Projector::new(Alice, transport);
projector.epp_and_run(HelloWorldChoreography);
```
