# Projector

Projector is responsible for performing the end-point projection and executing the choreography.

## Creating a Projector

To create a `Projector`, you need to provide the set of locations it can work with, the target location, and the transport.

```rust
# extern crate chorus_lib;
# use chorus_lib::transport::local::LocalTransport;
# use chorus_lib::core::{ChoreographyLocation, Projector};
# use chorus_lib::{hlist, projector};
# let transport = LocalTransport::from(&[Alice::name(), Bob::name()]);
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
#

type AL = hlist!(Alice, Bob);
let projector = projector!(AL, Alice, transport);
```

Notice that the `Projector` is parameterized by the location type. You will need one projector for each location to execute choreography.

## Executing a Choreography

To execute a choreography, you need to call the `epp_and_run` method on the `Projector` instance. The `epp_and_run` method takes a choreography, performs the end-point projection, and executes the choreography.

```rust
# extern crate chorus_lib;
# use chorus_lib::transport::local::LocalTransport;
# use chorus_lib::core::{ChoreographyLocation, Projector, Choreography, ChoreoOp};
# use chorus_lib::{hlist, projector};
# let transport = LocalTransport::from(&[Alice::name(), Bob::name()]);
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#     }
# }
#

# type AL = hlist!(Alice);
# let projector = projector!(AL, Alice, transport);
projector.epp_and_run(HelloWorldChoreography);
```

If the choreography has a return value, the `epp_and_run` method will return the value. We will discuss the return values in the [Input and Output](./guide-input-and-output.md) section.

### Note on the available location set of the Choreography

Keep in mind that when calling `epp_and_run`, you will get a compile error if the set of available locations of the `Choreography` is not a subset of the available locations of the `Projector`. In other words, the `Projector` should be allowed to do end-point projection into every `ChoreographyLocation` there is in the `Choreography`. So this will fail:

```rust, compile_fail
# extern crate chorus_lib;
# use chorus_lib::transport::local::LocalTransport;
# use chorus_lib::core::{ChoreographyLocation, Projector, Choreography, ChoreoOp};
# use chorus_lib::{hlist, projector};
# let transport = LocalTransport::from(&[Alice::name(), Bob::name()]);
# #[derive(ChoreographyLocation)]
# struct Alice;
# #[derive(ChoreographyLocation)]
# struct Bob;
struct HelloWorldChoreography;
impl Choreography for HelloWorldChoreography {
     type L = hlist!(Alice, Bob);
     fn run(self, op: &impl ChoreoOp<Self::L>) {
     }
}


type AL = hlist!(Alice);
let projector = projector!(AL, Alice, transport);
projector.epp_and_run(HelloWorldChoreography);
```
