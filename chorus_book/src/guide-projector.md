# Projector

Projector is responsible for performing the end-point projection and executing the choreography.

## Creating a Projector

To create a `Projector`, you need to provide the target location and the transport.

```rust
# extern crate chorus_lib;
# use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};
# use chorus_lib::core::{ChoreographyLocation, Projector};
# use chorus_lib::{LocationSet};
# let transport_channel = LocalTransportChannel::<LocationSet!(Alice, Bob)>::new();
# let alice_transport = LocalTransport::new(Alice, transport_channel.clone());
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
#

let projector = Projector::new(Alice, alice_transport);
```

Notice that the `Projector` is parameterized by its target location type. You will need one projector for each location to execute choreography.

## Executing a Choreography

To execute a choreography, you need to call the `epp_and_run` method on the `Projector` instance. The `epp_and_run` method takes a choreography, performs the end-point projection, and executes the choreography.

```rust
# extern crate chorus_lib;
# use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};
# use chorus_lib::core::{ChoreographyLocation, Projector, Choreography, ChoreoOp};
# use chorus_lib::{LocationSet};
# let transport_channel = LocalTransportChannel::<LocationSet!(Alice, Bob)>::new();
# let alice_transport = LocalTransport::new(Alice, transport_channel.clone());
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = LocationSet!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#     }
# }
#

# let projector = Projector::new(Alice, alice_transport);
projector.epp_and_run(HelloWorldChoreography);
```

If the choreography has a return value, the `epp_and_run` method will return the value. We will discuss the return values in the [Input and Output](./guide-input-and-output.md) section.
