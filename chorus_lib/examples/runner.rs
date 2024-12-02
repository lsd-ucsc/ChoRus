extern crate chorus_lib;
use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, Located, LocationSet, Runner,
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

struct BobCarolResult {
    is_even_at_bob: Located<bool, Bob>,
    is_even_at_carol: Located<bool, Carol>,
}

struct BobCarolChoreography {
    x_at_bob: Located<u32, Bob>,
}

impl Choreography<BobCarolResult> for BobCarolChoreography {
    type L = LocationSet!(Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> BobCarolResult {
        let is_even_at_bob: Located<bool, Bob> = op.locally(Bob, |un| {
            let x = un.unwrap(&self.x_at_bob);
            x % 2 == 0
        });
        let is_even: bool = op.broadcast(Bob, is_even_at_bob.clone());
        if is_even {
            let x_at_carol = op.comm(Bob, Carol, &self.x_at_bob);
            op.locally(Carol, |un| {
                let x = un.unwrap(&x_at_carol);
                println!("x is even: {}", *x);
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
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let x_at_alice = op.locally(Alice, |_| get_random_number());
        let x_at_bob = op.comm(Alice, Bob, &x_at_alice);
        let result = op.enclave(BobCarolChoreography { x_at_bob });
        op.locally(Bob, |un| {
            let is_even = un.unwrap(&un.unwrap(&result).is_even_at_bob);
            assert!(is_even);
            println!("Bob: x is even: {}", is_even);
        });
        op.locally(Carol, |un| {
            let is_even = un.unwrap(&un.unwrap(&result).is_even_at_carol);
            assert!(is_even);
            println!("Carol: x is even: {}", is_even);
        });
    }
}

fn main() {
    let runner = Runner::new();
    runner.run(MainChoreography);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_test() {
        let runner = Runner::new();
        runner.run(MainChoreography);
    }
}
