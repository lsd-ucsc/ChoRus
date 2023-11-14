# Choreographic Enclave and Efficient Conditional

ChoRus supports the `enclave` operator to achieve efficient conditional execution.

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
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let x_at_alice = op.locally(Alice, |_| {
            get_random_number()
        });
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(&x_at_bob);
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob);
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(&x_at_carol);
                println!("x is even: {}", x);
            });
        }
    }
}
```

While this code correctly implements the protocol, it is inefficient. The `is_even` value is broadcasted to all locations, but Alice does not need to receive the value. Ideally, we want to send `is_even_at_bob` only to Carol and branch only on Bob and Carol.

In ChoRus, we can achieve this using the `enclave` operator. First, let us define a sub-choreography that describes the communication between Bob and Carol:

```rust
{{#include ./header.txt}}
struct BobCarolChoreography {
    x_at_bob: Located<u32, Bob>,
};
impl Choreography for BobCarolChoreography {
    type L = LocationSet!(Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(&self.x_at_bob);
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob);
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(&x_at_carol);
                println!("x is even: {}", x);
            });
        }
    }
}
```

Notice that `BobCarolChoreography` only describes the behavior of Bob and Carol (see its location set `L`). `enclave` is an operator to execute a choreography only at locations that is included in the location set. In this case, if we invoke `BobCarolChoreography` with `enclave` in the main choreography, it will only be executed at Bob and Carol and not at Alice.

```rust
{{#include ./header.txt}}
# fn get_random_number() -> u32 {
#   42 // for presentation purpose
# }
# struct BobCarolChoreography {
#     x_at_bob: Located<u32, Bob>,
# };
# impl Choreography for BobCarolChoreography {
#     type L = LocationSet!(Bob, Carol);
#     fn run(self, op: &impl ChoreoOp<Self::L>) {
#         let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
#             let x = un.unwrap(&self.x_at_bob);
#             x % 2 == 0
#         });
#         let is_even: bool = op.broadcast(Bob, is_even_at_bob);
#         if is_even {
#             let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
#             op.locally(Carol, |un| {
#                 let x = un.unwrap(&x_at_carol);
#                 println!("x is even: {}", x);
#             });
#         }
#     }
# }
struct MainChoreography;
impl Choreography for MainChoreography {
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let x_at_alice = op.locally(Alice, |_| {
            get_random_number()
        });
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        op.enclave(BobCarolChoreography {
            x_at_bob,
        });
    }
}
```

## Returning Values from Enclave

Just like the `call` operator, the `enclave` operator can return a value. However, the type of the returned value must implement the `Superposition` trait. `Superposition` provides a way for ChoRus to construct a value on locations that are not specified in the `enclave` operator.

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
    type L = LocationSet!(Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> BobCarolResult {
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(&self.x_at_bob);
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob.clone());
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(&x_at_carol);
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
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let x_at_alice = op.locally(Alice, |_| {
            get_random_number()
        });
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        let BobCarolResult {
            is_even_at_bob,
            is_even_at_carol,
        } = op.enclave(BobCarolChoreography {
            x_at_bob,
        });
        // can access is_even_at_bob and is_even_at_carol using `locally` on Bob and Carol
    }
}
```
