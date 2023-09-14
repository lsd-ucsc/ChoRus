extern crate chorus_lib;
use chorus_lib::{core::{ChoreoOp, Choreography, ChoreographyLocation, Located}, hlist};
#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

struct DemoChoreography {
    input: Located<String, Alice>,
}

impl Choreography for DemoChoreography {
    type L = hlist!(Alice);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        op.locally(Alice, |un| {
            let s = un.unwrap(&self.input);
            println!("Alice received: {}", s);
        });
    }
}

fn main() {
    let runner = chorus_lib::core::Runner::new();
    let s = "hello, world".to_string();
    let s = runner.local(s);
    let choreo = DemoChoreography { input: s };
    runner.run(choreo);
}
