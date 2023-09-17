extern crate chorus_lib;
use std::fmt::Debug;
use std::thread;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, Located, Portable};
use chorus_lib::transport::local::LocalTransport;
use chorus_lib::{projector, LocationSet};

#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

struct CommAndPrint<V: Portable, L1: ChoreographyLocation, L2: ChoreographyLocation> {
    sender: L1,
    receiver: L2,
    data: Located<V, L1>,
}

impl<V, L1, L2> Choreography<Located<V, L2>> for CommAndPrint<V, L1, L2>
where
    V: Portable + Debug,
    L1: ChoreographyLocation,
    L2: ChoreographyLocation,
{
    type L = LocationSet!(L1, L2);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<V, L2> {
        let v = op.comm(self.sender, self.receiver, &self.data);
        op.locally(self.receiver, |un| println!("{:?}", un.unwrap(&v)));
        v
    }
}

struct MainChoreography;

impl Choreography<Located<i32, Alice>> for MainChoreography {
    type L = LocationSet!(Alice, Bob);

    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<i32, Alice> {
        let v1 = op.locally(Alice, |_| 100);
        let v2 = op.call(CommAndPrint {
            sender: Alice,
            receiver: Bob,
            data: v1,
        });
        let v2 = op.locally(Bob, |un| un.unwrap(&v2) + 10);
        return op.colocally(CommAndPrint {
            sender: Bob,
            receiver: Alice,
            data: v2,
        });
    }
}

fn main() {
    let transport = LocalTransport::from(&[Alice::name(), Bob::name(), Carol::name()]);
    let mut handles = vec![];
    {
        let transport = transport.clone();
        handles.push(thread::spawn(|| {
            let p = projector!(LocationSet!(Alice, Bob), Alice, transport);
            let v = p.epp_and_run(MainChoreography);
            assert_eq!(p.unwrap(v), 110);
        }));
    }
    {
        let transport = transport.clone();
        handles.push(thread::spawn(|| {
            let p = projector!(LocationSet!(Alice, Bob), Bob, transport);
            p.epp_and_run(MainChoreography);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
