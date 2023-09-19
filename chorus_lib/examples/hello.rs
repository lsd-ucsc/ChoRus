extern crate chorus_lib;

use std::thread;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, Projector};
use chorus_lib::transport::local::LocalTransport;
use chorus_lib::transport_config;
use chorus_lib::LocationSet;

// --- Define two locations (Alice and Bob) ---

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

// --- Define a choreography ---
struct HelloWorldChoreography;

// Implement the `Choreography` trait for `HelloWorldChoreography`
impl Choreography for HelloWorldChoreography {
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
    let transport_channel = LocalTransport::<LocationSet!(Alice, Bob)>::transport_channel();

    // Run the choreography in two threads
    {
        let config = transport_config!(
            Alice,
            Alice: (),
            Bob: ()
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);
        handles.push(thread::spawn(move || {
            let p = Projector::new(Alice, transport);
            p.epp_and_run(HelloWorldChoreography);
        }));
    }
    {
        let config = transport_config!(
            Alice,
            Alice: (),
            Bob: ()
        );

        let transport_channel = transport_channel.clone();
        let transport = LocalTransport::new(&config, transport_channel);

        handles.push(thread::spawn(move || {
            let p = Projector::new(Bob, transport);
            p.epp_and_run(HelloWorldChoreography);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
