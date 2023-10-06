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

A `LocationSet` is a special type representing a set of `ChoreographyLocation` types. It's used to ensure type safety within the system, and you'll see its application in future sections. To build a `LocationSet` type, you can use the `LocationSet` macro from the `chorus_lib` crate.

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
