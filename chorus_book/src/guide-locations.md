# Locations

Before we can start writing choreographies, we need to define _locations_. A location is a place where a choreography can be executed. A location can be a physical location, such as a computer, or a logical location, such as a thread.

To define a location, we need to create a struct and derive the `ChoreographyLocation` trait.

```rust
# extern crate chorus_lib;
use chorus_lib::core::ChoreographyLocation;

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;
```

The `ChoreographyLocation` trait provides the `name` method, which returns the name of the location as a `&'static str`. The name of a location is used to identify the location when performing end-point projection.

```rust,ignore
# use chorus_lib::core::ChoreographyLocation;
#
# #[derive(ChoreographyLocation)]
# struct Alice;
#
# #[derive(ChoreographyLocation)]
# struct Bob;
#
let name = Alice::name();
assert_eq!(name, "Alice");
```

## Location Set

Each Choreography is allowed to operate on a set of `ChoreographyLocation`, called its `LocationSet`. You can use the macro `LocationSet!` and give it a comma separated list of `ChoreographyLocation` to build a `LocationSet`.

```rust
# extern crate chorus_lib;
# use chorus_lib::core::ChoreographyLocation;
# #[derive(ChoreographyLocation)]
# struct Alice;
#
# #[derive(ChoreographyLocation)]
# struct Bob;
use chorus_lib::core::LocationSet;

type L = LocationSet!(Alice, Bob);
```

Internally, `LocationSet` is also used at other places like [Projector](./guide-projector.md) and [Transport](./guide-transport.md) to ensure that they have comprehensive information regarding the `ChoreographyLocation` values they're working with. This is crucial as it allows the system to catch potential errors during compile time instead of runtime, leading to safer code. You can check the API documentation for more details.
