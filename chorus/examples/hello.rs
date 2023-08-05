extern crate chorus;

use std::thread;

use chorus::core::{ChoreoOp, Choreography, ChoreographyLocation, Projector};
use chorus::transport::local::LocalTransport;

use rand::Rng;

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

struct HelloWorldChoreography;

// Implement the `Choreography` trait for `HelloWorldChoreography`
impl Choreography for HelloWorldChoreography {
    fn run(&self, op: &impl ChoreoOp) {
        let msg_at_alice = op.locally(Alice, |_| {
            println!("Hello from Alice!");
            let coin = rand::thread_rng().gen_bool(0.5);
            coin
        });
        let msg_at_bob = op.comm(Alice, Bob, &msg_at_alice);
        let msg_at_bob = op.locally(Bob, |un| {
            let msg = un.unwrap(&msg_at_bob);
            println!("Bob received a message: {}", msg);
            msg
        });
        let coin = op.broadcast(Bob, &msg_at_bob);
        if coin {
            println!("TRUE");
        }
    }
}

fn main() {
    let transport = LocalTransport::from(&[Alice.name(), Bob.name()]);
    let alice_transport = transport.clone();
    let bob_transport = transport.clone();

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(|| {
        let p = Projector::new(Alice, alice_transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
    handles.push(thread::spawn(|| {
        let p = Projector::new(Bob, bob_transport);
        p.epp_and_run(HelloWorldChoreography);
    }));
    for h in handles {
        h.join().unwrap();
    }
}
