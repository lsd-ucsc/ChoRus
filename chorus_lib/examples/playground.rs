use std::env;
use std::io::{self, Write};

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
/// cargo run --example playground <role>
/// where <role> is either "alpha" or "beta".
///
use chorus_lib::{
    core::{Choreography, ChoreographyLocation, LocationSet, Projector},
    transport::http::{HttpTransport, HttpTransportConfigBuilder},
};

// STEP 1: Add locations
#[derive(ChoreographyLocation)]
struct Alpha;

#[derive(ChoreographyLocation)]
struct Beta;

// STEP 2: Write a Choreography
struct MainChoreography;

impl Choreography for MainChoreography {
    type L = LocationSet!(Alpha, Beta);

    fn run(self, op: &impl chorus_lib::core::ChoreoOp<Self::L>) -> () {
        let a = op.locally(Alpha, |_| loop {
            print!("Enter a number: ");
            io::stdout().flush().expect("Failed to flush stdout");
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                continue;
            }
            if let Ok(num) = input.trim().parse::<i32>() {
                break num;
            } else {
                println!("Please enter a valid integer.");
            }
        });
        let a = op.comm(Alpha, Beta, &a);
        let b = op.locally(Beta, |_| {
            print!("Enter a word for Beta to send to Alpha: ");
            io::stdout().flush().expect("Failed to flush stdout");
            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to read line");
            input.trim().to_string()
        });
        let b = op.comm(Beta, Alpha, &b);
        op.locally(Alpha, |un| {
            println!("Alpha received: {}", un.unwrap(&b));
        });
        op.locally(Beta, |un| {
            println!("Beta received: {}", un.unwrap(&a));
        });
    }
}

// STEP 3: Run the choreography
fn main() {
    let role = env::args().nth(1).expect("Usage: playground <role>");
    match role.as_str() {
        "alpha" => {
            let config = HttpTransportConfigBuilder::for_target(Alpha, ("0.0.0.0", 8080))
                .with(Beta, ("127.0.0.1", 8081))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Alpha, transport);
            projector.epp_and_run(MainChoreography);
        }
        "beta" => {
            let config = HttpTransportConfigBuilder::for_target(Beta, ("0.0.0.0", 8081))
                .with(Alpha, ("127.0.0.1", 8080))
                .build();
            let transport = HttpTransport::new(config);
            let projector = Projector::new(Beta, transport);
            projector.epp_and_run(MainChoreography);
        }
        _ => panic!("Invalid role"),
    };
}
