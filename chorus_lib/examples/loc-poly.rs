extern crate chorus_lib;
use std::marker::PhantomData;
use std::thread;

use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, HList, Member, Projector};
use chorus_lib::hlist;
use chorus_lib::transport::local::LocalTransport;

#[derive(ChoreographyLocation)]
struct Alice;
#[derive(ChoreographyLocation)]
struct Bob;
#[derive(ChoreographyLocation)]
struct Carol;

struct LocationPolymorphicChoreography<L: HList, L1: ChoreographyLocation, Index>
where
    L1: Member<L, Index>,
{
    index: PhantomData<Index>,
    phantom: PhantomData<L>,
    location: L1,
}

impl<L: HList, L1: ChoreographyLocation, Index> Choreography
    for LocationPolymorphicChoreography<L, L1, Index>
where
    L1: Member<L, Index>,
{
    type L = L;
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        op.locally(self.location, |_| {
            println!("Hello, World at {:?}", L1::name());
        });
    }
}

fn main() {
    let transport = LocalTransport::from(&[Alice::name(), Bob::name(), Carol::name()]);

    let mut handles = vec![];
    {
        let transport = transport.clone();
        let alice_say_hello: LocationPolymorphicChoreography<hlist!(Alice), Alice, _> =
            LocationPolymorphicChoreography {
                location: Alice,
                phantom: PhantomData,
                index: PhantomData,
            };
        handles.push(thread::spawn(|| {
            let p = Projector::new(Alice, transport);
            p.epp_and_run(alice_say_hello);
        }));
    }
    {
        let transport = transport.clone();
        let bob_say_hello: LocationPolymorphicChoreography<hlist!(Bob), Bob, _> =
            LocationPolymorphicChoreography {
                location: Bob,
                phantom: PhantomData,
                index: PhantomData,
            };
        handles.push(thread::spawn(|| {
            let p = Projector::new(Bob, transport);
            p.epp_and_run(bob_say_hello);
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
}
