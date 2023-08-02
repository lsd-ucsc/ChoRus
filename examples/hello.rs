extern crate chorus;

use std::thread;

use chorus::backend::local::LocalBackend;
use chorus::core::{epp_and_run, ChoreoOp, Choreography, ChoreographyLocation};

use rand::Rng;

struct Alice;
impl ChoreographyLocation for Alice {
    fn name(&self) -> &'static str {
        "Alice"
    }
}
const ALICE: Alice = Alice;

struct Bob;

impl ChoreographyLocation for Bob {
    fn name(&self) -> &'static str {
        "Bob"
    }
}
const BOB: Bob = Bob;

struct HelloWorldChoreography;

// Implement the `Choreography` trait for `HelloWorldChoreography`
impl Choreography for HelloWorldChoreography {
    fn run(&self, op: &impl ChoreoOp) {
        let msg_at_alice = op.locally(ALICE, |_| {
            println!("Hello from Alice!");
            let coin = rand::thread_rng().gen_bool(0.5);
            coin
        });
        let msg_at_bob = op.comm(ALICE, BOB, msg_at_alice);
        let msg_at_bob = op.locally(BOB, |un| {
            let msg = un.unwrap(&msg_at_bob);
            println!("Bob received a message: {}", msg);
            msg
        });
        let coin = op.broadcast(BOB, msg_at_bob);
        if coin {
            println!("TRUE");
        }
    }
}

fn main() {
    let backend = LocalBackend::from(vec!["Alice", "Bob"].into_iter());
    let alice_backend = backend.clone();
    let bob_backend = backend.clone();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(|| {
        epp_and_run(HelloWorldChoreography, ALICE, alice_backend);
    }));
    handles.push(thread::spawn(|| {
        epp_and_run(HelloWorldChoreography, BOB, bob_backend);
    }));
    for h in handles {
        h.join().unwrap();
    }
}
