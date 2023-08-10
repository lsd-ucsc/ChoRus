extern crate chorus_lib;
use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, Located};
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
    fn run(self, op: &impl ChoreoOp) {
        op.locally(Alice, |un| {
            let s = un.unwrap(self.input.clone());
        });
        op.locally(Alice, |un| {
            let s = un.unwrap(self.input);
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
