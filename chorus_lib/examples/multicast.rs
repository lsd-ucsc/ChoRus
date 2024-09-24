extern crate chorus_lib;

use std::thread;

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, LocationSet, MulticastBuilder, Projector,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

// --- Define two locations (Alice and Bob) ---

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

#[derive(ChoreographyLocation)]
struct Carol;

// --- Define a choreography ---
struct MulticastChoreography;

// Implement the `Choreography` trait for `HelloWorldChoreography`
impl Choreography for MulticastChoreography {
    // Define the set of locations involved in the choreography.
    // In this case, the set consists of `Alice` and `Bob` and
    // the choreography can use theses locations.
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        // Create a located value at Alice
        let msg_at_alice = op.locally(Alice, |_| {
            println!("Hello from Alice!");
            "Hello from Alice!".to_string()
        });
        let msg_at_bob_and_carol =
            op.multicast(MulticastBuilder::new(Alice, msg_at_alice).to(Bob).to(Carol));
        op.locally(Bob, |un| {
            let msg = un.unwrap(&msg_at_bob_and_carol);
            println!("Bob received: {}", msg);
        });
        op.locally(Carol, |un| {
            let msg = un.unwrap(&msg_at_bob_and_carol);
            println!("Carol received: {}", msg);
        });
    }
}

fn main() {
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    // Create a transport channel
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .with(Carol)
        .build();
    // Run the choreography in two threads
    {
        let transport = LocalTransport::new(Alice, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let p = Projector::new(Alice, transport);
            p.epp_and_run(MulticastChoreography);
        }));
    }
    {
        let transport = LocalTransport::new(Bob, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let p = Projector::new(Bob, transport);
            p.epp_and_run(MulticastChoreography);
        }));
    }
    {
        let transport = LocalTransport::new(Carol, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let p = Projector::new(Carol, transport);
            p.epp_and_run(MulticastChoreography);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
