# Choreography

Choreography is a program that describes the behavior of a distributed system as a whole. To define a choreography, create a struct and implement the `Choreography` trait.

```rust
{{#include ./header.txt}}
// 1. Define a struct
struct HelloWorldChoreography;

// 2. Implement the `Choreography` trait
impl Choreography for HelloWorldChoreography {
    type L = hlist!(Alice);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        // 3. Use the `op` parameter to access operators
        op.locally(Alice, |_| {
            println!("Hello, World!");
        });
    }
}
```

`Choreography` must implement the `run` method which defines the behavior of the system. The `run` method takes a reference to an object that implements the `ChoreoOp` trait. The `ChoreoOp` trait provides choreographic operators such as `locally` and `comm`.

Also, each `Choreography` has an associated type `L`, which is the set of `ChoreographyLocation`s it can operate on. To build a set of locations, you can use the macro `hlist!`.

## Choreographic Operators

Inside the `run` method, you can use the `op` parameter to access choreographic operators.

### `locally`

The `locally` operator is used to perform a computation at a single location. It takes two parameters: a location and a closure. The closure is executed only at the specified location.

```rust
{{#include ./header.txt}}
#
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
op.locally(Alice, |_| {
    println!("Hello, World!");
});
#     }
# }
```

The closure can return a value to create a located value. Located values are values that are only available at a single location. When the computation closure returns a located value, the `locally` operator returns a located value at the same location.

```rust
{{#include ./header.txt}}
#
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
// This value is only available at Alice
let num_at_alice: Located<i32, Alice> = op.locally(Alice, |_| {
    42
});
#     }
# }
```

The computation closure takes `Unwrapper`. Using the `Unwrapper`, you can get a reference out of a located value.

```rust
{{#include ./header.txt}}
#
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
let num_at_alice: Located<i32, Alice> = op.locally(Alice, |_| {
    42
});
op.locally(Alice, |un| {
    let num: &i32 = un.unwrap(&num_at_alice);
    println!("The number at Alice is {}", num);
    assert_eq!(*num, 42);
});
#     }
# }
```

Note that you can unwrap a located value only at the location where the located value is available. If you try to unwrap a located value at a different location, the program will fail to compile.

```rust, compile_fail
{{#include ./header.txt}}
#
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice, Bob);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
// This code will fail to compile
let num_at_alice = op.locally(Alice, |_| { 42 });
op.locally(Bob, |un| {
    // Only values located at Bob can be unwrapped here
    let num_at_alice: &i32 = un.unwrap(&num_at_alice);
});
#     }
# }
```

We will discuss located values in more detail in the [Located Values](./guide-located-values.md) section.

### `comm`

The `comm` operator is used to perform a communication between two locations. It takes three parameters: a source location, a destination location, and a located value at the source location. The located value is sent from the source location to the destination location, and the operator returns a located value at the destination location.

```rust
{{#include ./header.txt}}
#
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice, Bob);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
// This value is only available at Alice
let num_at_alice: Located<i32, Alice> = op.locally(Alice, |_| {
    42
});
// Send the value from Alice to Bob
let num_at_bob: Located<i32, Bob> = op.comm(Alice, Bob, &num_at_alice);
// Bob can now access the value
op.locally(Bob, |un| {
    let num_at_bob: &i32 = un.unwrap(&num_at_bob);
    println!("The number at Bob is {}", num_at_bob);
});
#     }
# }
```

### `broadcast`

The `broadcast` operator is used to perform a broadcast from a single location to multiple locations. It takes two parameters: a source location and a located value at the source location. The located value is sent from the source location to all other locations, and the operator returns a normal value.

```rust
{{#include ./header.txt}}
#
# struct HelloWorldChoreography;
# impl Choreography for HelloWorldChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
// This value is only available at Alice
let num_at_alice: Located<i32, Alice> = op.locally(Alice, |_| {
    42
});
// Broadcast the value from Alice to all other locations
let num: i32 = op.broadcast(Alice, num_at_alice);
#     }
# }
```

Because all locations receive the value, the return type of the `broadcast` operator is a normal value, not a located value. This means that the value can be used for control flow.

```rust, ignore
if num == 42 {
    println!("The number is 42!");
} else {
    println!("The number is not 42!");
}
```

### Note on invalid values for Choreography::L

You'll get a compile error if you try to work with a `ChoreographyLocation` that is not a member of `L`.

```rust, compile_fail
# {{#include ./header.txt}}
# // 1. Define a struct
# struct HelloWorldChoreography;

# // 2. Implement the `Choreography` trait
// ...
impl Choreography for HelloWorldChoreography {
    type L = hlist!(Alice);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        // this will fail
        op.locally(Bob, |_| {
            println!("Hello, World!");
        });
    }
}
```

