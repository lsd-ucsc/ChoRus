extern crate chorus_lib;
use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, Located, Runner, Superposition,
};
#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

fn get_random_number() -> u32 {
    42 // for presentation purpose
}

#[derive(Superposition)]
struct BobCarolResult {
    is_even_at_bob: Located<bool, Bob>,
    is_even_at_carol: Located<bool, Carol>,
}

struct BobCarolChoreography {
    x_at_bob: Located<u32, Bob>,
}

impl Choreography<BobCarolResult> for BobCarolChoreography {
    fn run(&self, op: &impl ChoreoOp) -> BobCarolResult {
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(&self.x_at_bob);
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, &is_even_at_bob);
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(&x_at_carol);
                println!("x is even: {}", x);
            });
        }
        BobCarolResult {
            is_even_at_bob,
            is_even_at_carol: op.locally(Carol, |_| is_even),
        }
    }
}

struct MainChoreography;

impl Choreography for MainChoreography {
    fn run(&self, op: &impl ChoreoOp) {
        let x_at_alice = op.locally(Alice, |_| get_random_number());
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        let BobCarolResult {
            is_even_at_bob,
            is_even_at_carol,
        } = op.colocally(
            &[Bob.name(), Carol.name()],
            &BobCarolChoreography { x_at_bob },
        );
        op.locally(Bob, |un| {
            let is_even = un.unwrap(&is_even_at_bob);
            assert!(is_even);
            println!("Bob: x is even: {}", is_even);
        });
        op.locally(Carol, |un| {
            let is_even = un.unwrap(&is_even_at_carol);
            assert!(is_even);
            println!("Carol: x is even: {}", is_even);
        });
    }
}

fn main() {
    let runner = Runner::new();
    runner.run(&MainChoreography);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_test() {
        let runner = Runner::new();
        runner.run(&MainChoreography);
    }
}
