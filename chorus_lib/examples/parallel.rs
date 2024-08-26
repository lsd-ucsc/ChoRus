extern crate chorus_lib;

use std::thread;

use rand::Rng;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, LocationSet, Projector};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

#[derive(ChoreographyLocation, Debug)]
struct Alice;

#[derive(ChoreographyLocation, Debug)]
struct Bob;

#[derive(ChoreographyLocation, Debug)]
struct Carol;

struct ParallelChoreography;
impl Choreography for ParallelChoreography {
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let faceted = op.parallel(<LocationSet!(Alice, Bob, Carol)>::new(), || {
            // return a random number between 1 and 10
            rand::thread_rng().gen_range(1..=10)
        });
        op.locally(Alice, |un| {
            let x = un.unwrap3(&faceted);
            println!("Alice picked {}", x);
        });
        op.locally(Bob, |un| {
            let x = un.unwrap3(&faceted);
            println!("Bob picked {}", x);
        });
        op.locally(Carol, |un| {
            let x = un.unwrap3(&faceted);
            println!("Carol picked {}", x);
        });
    }
}

fn main() {
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .with(Carol)
        .build();
    let transport_alice = LocalTransport::new(Alice, transport_channel.clone());
    let transport_bob = LocalTransport::new(Bob, transport_channel.clone());
    let transport_carol = LocalTransport::new(Carol, transport_channel.clone());

    let alice_projector = Projector::new(Alice, transport_alice);
    let bob_projector = Projector::new(Bob, transport_bob);
    let carol_projector = Projector::new(Carol, transport_carol);

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(move || {
        alice_projector.epp_and_run(ParallelChoreography);
    }));
    handles.push(thread::spawn(move || {
        bob_projector.epp_and_run(ParallelChoreography);
    }));
    handles.push(thread::spawn(move || {
        carol_projector.epp_and_run(ParallelChoreography);
    }));
    for handle in handles {
        handle.join().unwrap();
    }
}
