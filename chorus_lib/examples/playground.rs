/// # Testing Playground
///
/// This is a place where you can write your own code to test ChoRus.
/// Add your test cases, examples, or any experimental code here to validate
/// and understand the functionality of the library.
///
/// ## How to Run
///
/// You can run this program by using the following command:
///
/// cargo run --example playground
use chorus_lib::{
    core::{Choreography, ChoreographyLocation, LocationSet},
    transport::local::{LocalTransport, LocalTransportChannelBuilder},
};
use rand;

// STEP 1: Add locations
#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

// STEP 2: Write a Choreography
struct MainChoreography;

impl Choreography for MainChoreography {
    type L = LocationSet!(Alice, Bob);

    fn run(self, op: &impl chorus_lib::core::ChoreoOp<Self::L>) -> () {
        let random_number_at_alice = op.locally(Alice, |_| {
            let random_number = rand::random::<u32>();
            println!("Random number at Alice: {}", random_number);
            random_number
        });
        let random_number_at_bob = op.comm(Alice, Bob, &random_number_at_alice);
        op.locally(Bob, |un| {
            let random_number = un.unwrap(&random_number_at_bob);
            println!("Random number at Bob: {}", random_number);
        });
    }
}

// STEP 3: Run the choreography
fn main() {
    // In this example, we use the local transport and run the choreography in two threads.
    // Refer to the documentation for more information on how to use other transports.
    let mut handles = Vec::new();
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .build();
    {
        let transport = LocalTransport::new(Alice, transport_channel.clone());
        handles.push(std::thread::spawn(move || {
            let projector = chorus_lib::core::Projector::new(Alice, transport);
            projector.epp_and_run(MainChoreography);
        }));
    }
    {
        let transport = LocalTransport::new(Bob, transport_channel.clone());
        handles.push(std::thread::spawn(move || {
            let projector = chorus_lib::core::Projector::new(Bob, transport);
            projector.epp_and_run(MainChoreography);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
