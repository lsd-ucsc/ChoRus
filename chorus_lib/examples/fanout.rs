extern crate chorus_lib;

use std::marker::PhantomData;
use std::thread;

use rand::Rng;

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, FanOutChoreography, HCons, HNil, Here, Located,
    LocationSet, LocationSetFoldable, Member, Projector, There,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

#[derive(ChoreographyLocation, Debug)]
struct Alice;

#[derive(ChoreographyLocation, Debug)]
struct Bob;

#[derive(ChoreographyLocation, Debug)]
struct Carol;

struct FanOut<L: LocationSet, QS: LocationSet> {
    phantom: PhantomData<(L, QS)>,
}

impl<L: LocationSet, QS: LocationSet> FanOutChoreography<String> for FanOut<L, QS> {
    type L = L;
    type QS = QS;
    fn new() -> Self {
        FanOut {
            phantom: PhantomData,
        }
    }
    fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
        self,
        op: &impl ChoreoOp<Self::L>,
    ) -> Located<String, Q>
    where
        Self::QS: chorus_lib::core::Subset<Self::L, QSSubsetL>,
        Q: Member<Self::L, QMemberL>,
        Q: Member<Self::QS, QMemberQS>,
    {
        op.locally(Q::new(), |_| String::from(Q::name()))
    }
}

struct ParallelChoreography;
impl Choreography for ParallelChoreography {
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        type L = LocationSet!(Alice, Bob, Carol);
        op.fanout(
            <LocationSet!(Bob, Carol)>::new(),
            FanOut {
                phantom: PhantomData,
            },
        );
    }
}

fn main() {
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .with(Carol)
        .build();
    let transport_alice = LocalTransport::new(Alice, transport_channel.clone());
    let transport_bob = LocalTransport::new(Bob, transport_channel.clone());
    let transport_carol = LocalTransport::new(Carol, transport_channel.clone());

    let alice_projector = Projector::new(Alice, transport_alice);
    let bob_projector = Projector::new(Bob, transport_bob);
    let carol_projector = Projector::new(Carol, transport_carol);

    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    handles.push(thread::spawn(move || {
        alice_projector.epp_and_run(ParallelChoreography);
    }));
    handles.push(thread::spawn(move || {
        bob_projector.epp_and_run(ParallelChoreography);
    }));
    handles.push(thread::spawn(move || {
        carol_projector.epp_and_run(ParallelChoreography);
    }));
    for handle in handles {
        handle.join().unwrap();
    }
}
