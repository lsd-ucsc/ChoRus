extern crate chorus;

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

use chorus::core::{ChoreoOp, Choreography, ChoreographyLocation, Projector};
use chorus::transport::http::HttpTransport;

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
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    let config = Arc::new(HashMap::from([
        (Alice.name(), ("0.0.0.0", 8000)),
        (Bob.name(), ("0.0.0.0", 8001)),
    ]));
    {
        let config = config.clone();
        handles.push(thread::spawn(move || {
            let http_alice = HttpTransport::new(Alice.name(), &config);
            let p = Projector::new(Alice, http_alice);
            p.epp_and_run(HelloWorldChoreography);
        }));
    }
    {
        let config = config.clone();
        handles.push(thread::spawn(move || {
            let http_bob = HttpTransport::new(Bob.name(), &config);
            let p = Projector::new(Bob, http_bob);
            p.epp_and_run(HelloWorldChoreography);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
