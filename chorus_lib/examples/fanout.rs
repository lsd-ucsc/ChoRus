extern crate chorus_lib;

use std::marker::PhantomData;
use std::thread;

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, FanOutChoreography, Located, LocationSet, Member,
    Projector, Subset,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

#[derive(ChoreographyLocation, Debug)]
struct Alice;

#[derive(ChoreographyLocation, Debug)]
struct Bob;

#[derive(ChoreographyLocation, Debug)]
struct Carol;

struct FanOut<L: LocationSet, QS: LocationSet, Alice: ChoreographyLocation, AliceMemberL>
where
    Alice: Member<L, AliceMemberL>,
{
    phantom: PhantomData<(L, QS, Alice, AliceMemberL)>,
}

impl<L: LocationSet, QS: LocationSet, Alice: ChoreographyLocation, AliceMemberL>
    FanOut<L, QS, Alice, AliceMemberL>
where
    Alice: Member<L, AliceMemberL>,
{
    fn new(_: Alice) -> Self
    where
        Alice: Member<L, AliceMemberL>,
    {
        FanOut {
            phantom: PhantomData,
        }
    }
}

impl<L: LocationSet, QS: LocationSet, Alice: ChoreographyLocation, AliceMemberL>
    FanOutChoreography<String> for FanOut<L, QS, Alice, AliceMemberL>
where
    Alice: Member<L, AliceMemberL>,
{
    type L = L;
    type QS = QS;
    fn run<Q: ChoreographyLocation, QSSubsetL, QMemberL, QMemberQS>(
        &self,
        op: &impl ChoreoOp<Self::L>,
    ) -> Located<String, Q>
    where
        Self::QS: Subset<Self::L, QSSubsetL>,
        Q: Member<Self::L, QMemberL>,
        Q: Member<Self::QS, QMemberQS>,
    {
        let msg_at_alice = op.locally(Alice::new(), |_| {
            format!("{} says hi to {}", Alice::name(), Q::name())
        });
        let msg_at_q = op.comm(Alice::new(), Q::new(), &msg_at_alice);
        op.locally(Q::new(), |un| {
            println!("{} received: \"{}\"", Q::name(), un.unwrap(&msg_at_q))
        });
        msg_at_q
    }
}

struct ParallelChoreography;
impl Choreography for ParallelChoreography {
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let v = op.fanout(<LocationSet!(Bob, Carol)>::new(), FanOut::new(Bob));
        op.locally(Bob, |un| {
            println!("{}", un.unwrap3(&v));
        });
        op.locally(Carol, |un| {
            println!("{}", un.unwrap3(&v));
        });
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
    handles.push(
        thread::Builder::new()
            .name("Alice".to_string())
            .spawn(move || {
                alice_projector.epp_and_run(ParallelChoreography);
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Bob".to_string())
            .spawn(move || {
                bob_projector.epp_and_run(ParallelChoreography);
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Carol".to_string())
            .spawn(move || {
                carol_projector.epp_and_run(ParallelChoreography);
            })
            .unwrap(),
    );
    for handle in handles {
        handle.join().unwrap();
    }
}
