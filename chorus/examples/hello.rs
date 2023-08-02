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

struct Bob;
impl ChoreographyLocation for Bob {
    fn name(&self) -> &'static str {
        "Bob"
    }
}

struct HelloWorldChoreography;

// Implement the `Choreography` trait for `HelloWorldChoreography`
impl Choreography for HelloWorldChoreography {
    fn run(&self, op: &impl ChoreoOp) {
        let msg_at_alice = op.locally(Alice, |_| {
            println!("Hello from Alice!");
            let coin = rand::thread_rng().gen_bool(0.5);
            coin
        });
        let msg_at_bob = op.comm(Alice, Bob, msg_at_alice);
        let msg_at_bob = op.locally(Bob, |un| {
            let msg = un.unwrap(&msg_at_bob);
            println!("Bob received a message: {}", msg);
            msg
        });
        let coin = op.broadcast(Bob, msg_at_bob);
        if coin {
            println!("TRUE");
        }
    }
}

fn main() {
    let backend = LocalBackend::from(Vec::from([Alice.name(), Bob.name()]).into_iter());
    let alice_backend = backend.clone();
    let bob_backend = backend.clone();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(|| {
        epp_and_run(HelloWorldChoreography, Alice, alice_backend);
    }));
    handles.push(thread::spawn(|| {
        epp_and_run(HelloWorldChoreography, Bob, bob_backend);
    }));
    for h in handles {
        h.join().unwrap();
    }
}