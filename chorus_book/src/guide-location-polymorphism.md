# Location Polymorphism

Another feature of ChoRus is _location polymorphism_. Location polymorphism allows choreographies to be defined using generic locations. These locations can then be instantiated with concrete locations, allowing the choreography to be executed on different locations.

To define a location-polymorphic choreography, you need to create a generic struct that takes a type parameter that implements the `ChoreographyLocation` trait. When instantiating the choreography, you can pass a concrete location.

```rust
{{#include ./header.txt}}
struct LocationPolymorphicChoreography<L1: ChoreographyLocation> {
    location: L1,
}

impl<L1: ChoreographyLocation> Choreography for LocationPolymorphicChoreography<L1> {
    fn run(self, op: &impl ChoreoOp) {
        op.locally(self.location, |_| {
            println!("Hello, World!");
        });
    }
}

let alice_say_hello = LocationPolymorphicChoreography {
    location: Alice,
};
let bob_say_hello = LocationPolymorphicChoreography {
    location: Bob,
};
```
