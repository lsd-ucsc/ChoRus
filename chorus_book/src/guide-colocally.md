# colocally and Efficient Conditional

ChoRus supports the `colocally` operator to achieve efficient conditional execution.

## Conditional with Broadcast

Consider the following protocol:

1. Alice generates a random number `x` and sends it to Bob.
2. Bob checks if `x` is even. If it is even, Bob sends `x` to Carol. Otherwise, Bob terminates.

This protocol can be implemented as follows:

```rust
{{#include ./header.txt}}
# fn get_random_number() -> u32 {
#   42 // for presentation purpose
# }
#
struct DemoChoreography;

impl Choreography for DemoChoreography {
    fn run(self, op: &impl ChoreoOp) {
        let x_at_alice = op.locally(Alice, |_| {
            get_random_number()
        });
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(x_at_bob.clone());
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob);
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(x_at_carol);
                println!("x is even: {}", x);
            });
        }
    }
}
```

While this code correctly implements the protocol, it is inefficient. The `is_even` value is broadcasted to all locations, but Alice does not need to receive the value. Ideally, we want to send `is_even_at_bob` only to Carol and branch only on Bob and Carol.

In ChoRus, we can achieve this using the `colocally` operator. First, let us define a sub-choreography that describes the communication between Bob and Carol:

```rust
{{#include ./header.txt}}
struct BobCarolChoreography {
    x_at_bob: Located<u32, Bob>,
};
impl Choreography for BobCarolChoreography {
    fn run(self, op: &impl ChoreoOp) {
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(self.x_at_bob.clone());
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob);
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(x_at_carol);
                println!("x is even: {}", x);
            });
        }
    }
}
```

Notice that the `BobCarolChoreography` only describes the behavior of Bob and Carol. Since Alice does not appear in this choreography, we can use the `colocally` operator in the main choreography to execute the sub-choreography only on Bob and Carol.

```rust
{{#include ./header.txt}}
# fn get_random_number() -> u32 {
#   42 // for presentation purpose
# }
# struct BobCarolChoreography {
#     x_at_bob: Located<u32, Bob>,
# };
# impl Choreography for BobCarolChoreography {
#     fn run(self, op: &impl ChoreoOp) {
#         let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
#             let x = un.unwrap(self.x_at_bob.clone());
#             x % 2 == 0
#         });
#         let is_even: bool = op.broadcast(Bob, is_even_at_bob);
#         if is_even {
#             let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
#             op.locally(Carol, |un| {
#                 let x = un.unwrap(x_at_carol);
#                 println!("x is even: {}", x);
#             });
#         }
#     }
# }
struct MainChoreography;
impl Choreography for MainChoreography {
    fn run(self, op: &impl ChoreoOp) {
        let x_at_alice = op.locally(Alice, |_| {
            get_random_number()
        });
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        op.colocally(&[Bob.name(), Carol.name()], BobCarolChoreography {
            x_at_bob,
        });
    }
}
```

## Returning Values from Colocally

Just like the `call` operator, the `colocally` operator can return a value. However, the type of the returned value must implement the `Superposition` trait. `Superposition` provides a way for ChoRus to construct a value on locations that are not specified in the `colocally` operator.

In general, `Superposition` is either a located value or a struct consisting only of located values. The `Located` struct implements the `Superposition` trait, so you can return located values without any code. If you wish to return a struct of located values, you need to derive the `Superposition` trait using the derive macro.

```rust
{{#include ./header.txt}}
# fn get_random_number() -> u32 {
#   42 // for presentation purpose
# }
#
#[derive(Superposition)]
struct BobCarolResult {
    is_even_at_bob: Located<bool, Bob>,
    is_even_at_carol: Located<bool, Carol>,
}

struct BobCarolChoreography {
    x_at_bob: Located<u32, Bob>,
};

impl Choreography<BobCarolResult> for BobCarolChoreography {
    fn run(self, op: &impl ChoreoOp) -> BobCarolResult {
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(self.x_at_bob.clone());
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob.clone());
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(x_at_carol);
                println!("x is even: {}", x);
            });
        }
        BobCarolResult {
            is_even_at_bob,
            is_even_at_carol: op.locally(Carol, |_| is_even),
        }
    }
}

struct MainChoreography;

impl Choreography for MainChoreography {
    fn run(self, op: &impl ChoreoOp) {
        let x_at_alice = op.locally(Alice, |_| {
            get_random_number()
        });
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        let BobCarolResult {
            is_even_at_bob,
            is_even_at_carol,
        } = op.colocally(&[Bob.name(), Carol.name()], BobCarolChoreography {
            x_at_bob,
        });
        // can access is_even_at_bob and is_even_at_carol using `locally` on Bob and Carol
    }
}
```
