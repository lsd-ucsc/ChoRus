# Higher-order Choreography

_Higher-order choreography_ is a choreography that takes another choreography as an argument. Just like higher-order functions, higher-order choreographies are useful for abstracting over common patterns.

This section describes how to define and execute higher-order choreographies.

## Defining a Higher-order Choreography

To define a higher-order choreography, you need to create a generic struct that takes a type parameter that implements the `Choreography` trait.

```rust
{{#include ./header.txt}}
struct HigherOrderChoreography<C: Choreography> {
    sub_choreo: C,
};
```

When you implement the `Choreography` trait, you have access to the `sub_choreo` field. You can use the `call` method to execute the sub-choreography.

```rust
{{#include ./header.txt}}
# struct HigherOrderChoreography<C: Choreography> {
#     sub_choreo: C,
# };
impl<C: Choreography> Choreography for HigherOrderChoreography<C> {
    fn run(self, op: &impl ChoreoOp) {
        op.call(self.sub_choreo);
    }
}
```

## Passing values to a sub-choreography

It is often useful to pass values to a sub-choreography. To do so, instead of storing the sub-choreography object as a field, you associate the sub-choreography trait with the choreography using the `std::marker::PhantomData` type.

```rust
{{#include ./header.txt}}
use std::marker::PhantomData;

trait SubChoreography {
    fn new(arg: Located<i32, Alice>) -> Self;
}

struct HigherOrderChoreography<C: Choreography<Located<bool, Alice>> + SubChoreography> {
    _marker: PhantomData<C>,
};

impl<C: Choreography<Located<bool, Alice>> + SubChoreography> Choreography for HigherOrderChoreography<C> {
    fn run(self, op: &impl ChoreoOp) {
        let num_at_alice = op.locally(Alice, |_| {
            42
        });
        let sub_choreo = C::new(num_at_alice);
        op.call(sub_choreo);
    }
}
```

Here, the `HigherOrderChoreography` struct takes a type parameter `C` that implements both the `Choreography` trait and the `SubChoreography` trait. The `SubChoreography` trait ensures that the `C` type can be constructed with a located integer at Alice using the `new` constructor.
