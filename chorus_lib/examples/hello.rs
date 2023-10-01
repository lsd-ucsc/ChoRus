extern crate chorus_lib;

use std::thread;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, LocationSet, Projector};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannel};

// --- Define two locations (Alice and Bob) ---

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

// --- Define a choreography ---
struct HelloWorldChoreography;

// Implement the `Choreography` trait for `HelloWorldChoreography`
impl Choreography for HelloWorldChoreography {
    // Define the set of locations involved in the choreography.
    // In this case, the set consists of `Alice` and `Bob` and
    // the choreography can use theses locations.
    type L = LocationSet!(Alice, Bob);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        // Create a located value at Alice
        let msg_at_alice = op.locally(Alice, |_| {
            println!("Hello from Alice!");
            "Hello from Alice!".to_string()
        });
        // Send the located value to Bob
        let msg_at_bob = op.comm(Alice, Bob, &msg_at_alice);
        // Print the received message at Bob
        op.locally(Bob, |un| {
            let msg = un.unwrap(&msg_at_bob);
            println!("Bob received a message: {}", msg);
            msg
        });
    }
}

fn main() {
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    // Create a transport channel
    // let transport_channel = LocalTransportChannel::<LocationSet!(Bob, Alice)>::new();
    let transport_channel = LocalTransportChannel::new().with(Alice).with(Bob);
    // Run the choreography in two threads
    {
        // let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(Alice, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let p = Projector::new(Alice, transport);
            p.epp_and_run(HelloWorldChoreography);
        }));
    }
    {
        // let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(Bob, transport_channel.clone());
        handles.push(thread::spawn(move || {
            let p = Projector::new(Bob, transport);
            p.epp_and_run(HelloWorldChoreography);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
