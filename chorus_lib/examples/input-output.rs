extern crate chorus_lib;
use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, Located, LocationSet};
#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

struct DemoChoreography {
    input: Located<String, Alice>,
}

impl Choreography<Located<String, Alice>> for DemoChoreography {
    type L = LocationSet!(Alice);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<String, Alice> {
        op.locally(Alice, |un| {
            let s = un.unwrap(&self.input);
            println!("Alice received: {}", s);
        });
        op.locally(Alice, |_| "HELLO, WORLD".to_string())
    }
}

fn main() {
    let runner = chorus_lib::core::Runner::new();
    let s = "hello, world".to_string();
    let s = runner.local(s);
    let choreo = DemoChoreography { input: s };
    let result = runner.run(choreo);
    let result = runner.unwrap(result);
    println!("Returned value: {}", result);
}
