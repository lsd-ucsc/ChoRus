# Runner

`Runner` is a struct that can be used to run a `Choreography` without doing end-point projection. It gives semantics to the `Choreography` and allows it to be run in a way that is similar to a function call.

To use `Runner`, construct an instance using the `new` constructor, and then call the `run` method with the `Choreography`.

```rust
{{#include ./header.txt}}
# struct DemoChoreography;
# impl Choreography for DemoChoreography {
#     fn run(self, op: &impl ChoreoOp) {
#     }
# }
let runner = Runner::new();
runner.run(DemoChoreography);
```

As described in the [Input and Output](./guide-input-and-output.md) section, `Runner` can also pass values to the `Choreography` and receive values from it.

Because `Runner` executes the `Choreography` at all locations, all located inputs must be provided. Also, `Runner` can unwrap any located values returned by the `Choreography`.

```rust
{{#include ./header.txt}}
struct SumChoreography {
    x_at_alice: Located<u32, Alice>,
    y_at_bob: Located<u32, Bob>,
}
impl Choreography<Located<u32, Carol>> for SumChoreography {
    fn run(self, op: &impl ChoreoOp) -> Located<u32, Carol> {
        let x_at_carol = op.comm(Alice, Carol, &self.x_at_alice);
        let y_at_carol = op.comm(Bob, Carol, &self.y_at_bob);
        op.locally(Carol, |un| {
            let x = un.unwrap(x_at_carol);
            let y = un.unwrap(y_at_carol);
            x + y
        })
    }
}

let runner = Runner::new();
let x_at_alice = runner.local(1);
let y_at_bob = runner.local(2);
let sum_at_carol = runner.run(SumChoreography {
    x_at_alice,
    y_at_bob,
});
assert_eq!(runner.unwrap(sum_at_carol), 3);
```
