# Input and Output

To use ChoRus as part of a larger system, you need to be able to write a choreography that takes input and returns output.

Moreover, you may want to write a choreography that takes located values as input and returns located values as output. In such cases, you need to be able to construct/unwrap located values outside of the `run` method.

In this section, we will show you how to write a choreography that takes input and returns output.

## Input

To take input, you can use fields of the struct that implements the `Choreography` trait. For example, the following choreography takes a `String` as input.

```rust
{{#include ./header.txt}}
#
struct DemoChoreography {
    input: String,
}

impl Choreography for DemoChoreography {
    type L = hlist!();
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        println!("Input: {}", self.input);
    }
}
```

You can construct an instance of the choreography with the input and pass it to the `epp_and_run` function.

```rust
{{#include ./header.txt}}
# struct DemoChoreography {
#     input: String,
# }
# impl Choreography for DemoChoreography {
#     type L = hlist!();
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#         println!("Input: {}", self.input);
#     }
# }
#
let choreo = DemoChoreography {
    input: "World".to_string(),
};
let projector = Projector::new(Alice, transport);
projector.epp_and_run(choreo);
```

## Located Input

Input of normal types such as `String` must be available at all locations. However, you may want to take input that is only available at a single location. You can do so by using a located value as a field of the choreography struct.

```rust
{{#include ./header.txt}}
struct DemoChoreography {
    input: Located<String, Alice>,
}

impl Choreography for DemoChoreography {
    type L = hlist!(Alice);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        op.locally(Alice, |un| {
            let input = un.unwrap(&self.input);
            println!("Input at Alice: {}", input);
        });
    }
}
```

Because the `input` field is located at Alice, you can only access the string at Alice using the `Unwrapper`.

To construct this choreography, you must pass a located value. The `Projector` struct provides two methods to construct located values: `local` and `remote`.

`local` constructs a located value that is available at the projection target. You must provide an actual value as an argument. The location will be the same as the target of the projector.

`remote` constructs a located value that is available at a different location. You must provide a location of the value. Note that this location must be different from the target of the projector. As of now, ChoRus does not check this at compile time. If you pass the same location as the target of the projector, the program will panic at runtime.

To run the sample choreography above at Alice, we use the `local` method to construct the located value.

```rust
{{#include ./header.txt}}
# struct DemoChoreography {
#     input: Located<String, Alice>,
# }
#
# impl Choreography for DemoChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#         op.locally(Alice, |un| {
#             let input = un.unwrap(&self.input);
#             println!("Input at Alice: {}", input);
#         });
#     }
# }
let projector_for_alice = Projector::new(Alice, transport);
// Because the target of the projector is Alice, the located value is available at Alice.
let string_at_alice: Located<String, Alice> = projector_for_alice.local("Hello, World!".to_string());
// Instantiate the choreography with the located value
let choreo = DemoChoreography {
    input: string_at_alice,
};
projector_for_alice.epp_and_run(choreo);
```

For Bob, we use the `remote` method to construct the located value.

```rust
{{#include ./header.txt}}
# struct DemoChoreography {
#     input: Located<String, Alice>,
# }
#
# impl Choreography for DemoChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#         op.locally(Alice, |un| {
#             let input = un.unwrap(&self.input);
#             println!("Input at Alice: {}", input);
#         });
#     }
# }
let projector_for_bob = Projector::new(Bob, transport);
// Construct a remote located value at Alice. The actual value is not required.
let string_at_alice = projector_for_bob.remote(Alice);
// Instantiate the choreography with the located value
let choreo = DemoChoreography {
    input: string_at_alice,
};
projector_for_bob.epp_and_run(choreo);
```

## Output

Similarly, we can get output from choreographies by returning a value from the `run` method.

To do so, we specify the output type to the `Choreography` trait and return the value of the type from the `run` method.

```rust
{{#include ./header.txt}}
struct DemoChoreography;

impl Choreography<String> for DemoChoreography {
    type L = hlist!();
    fn run(self, op: &impl ChoreoOp<Self::L>) -> String {
        "Hello, World!".to_string()
    }
}
```

`epp_and_run` returns the value returned from the `run` method.

```rust
{{#include ./header.txt}}
# struct DemoChoreography;
#
# impl Choreography<String> for DemoChoreography {
#     type L = hlist!(Alice);
#     fn run(self, op: &impl ChoreoOp<Self::L>) -> String {
#         "Hello, World!".to_string()
#     }
# }
let choreo = DemoChoreography;
let projector = Projector::new(Alice, transport);
let output = projector.epp_and_run(choreo);
assert_eq!(output, "Hello, World!".to_string());
```

## Located Output

You can use the `Located<V, L1>` as a return type of the `run` method to return a located value. The projector provides a method `unwrap` to unwrap the output located values.

```rust
{{#include ./header.txt}}
struct DemoChoreography;

impl Choreography<Located<String, Alice>> for DemoChoreography {
    type L = hlist!(Alice);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<String, Alice> {
        op.locally(Alice, |_| {
            "Hello, World!".to_string()
        })
    }
}

let projector = Projector::new(Alice, transport);
let output = projector.epp_and_run(DemoChoreography);
let string_at_alice = projector.unwrap(output);
assert_eq!(string_at_alice, "Hello, World!".to_string());
```

Because projectors are parametric over locations, you can only unwrap located values at the target location.

You can return multiple located values by returning a tuple or struct that contains multiple located values. They don't have to be located at the same location, but you can only unwrap them at the correct location.
