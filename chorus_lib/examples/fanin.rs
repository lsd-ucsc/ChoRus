extern crate chorus_lib;

use std::marker::PhantomData;
use std::thread;

use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, FanInChoreography, Located, LocationSet, Member,
    MultiplyLocated, Projector, Quire, Subset,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};

#[derive(ChoreographyLocation, Debug)]
struct Alice;

#[derive(ChoreographyLocation, Debug)]
struct Bob;

#[derive(ChoreographyLocation, Debug)]
struct Carol;

struct FanIn<L: LocationSet, QS: LocationSet, Alice: ChoreographyLocation, AliceMemberL>
where
    Alice: Member<L, AliceMemberL>,
{
    phantom: PhantomData<(L, QS, Alice, AliceMemberL)>,
}

impl<L: LocationSet, QS: LocationSet, Alice: ChoreographyLocation, AliceMemberL>
    FanIn<L, QS, Alice, AliceMemberL>
where
    Alice: Member<L, AliceMemberL>,
{
    fn new(_: Alice) -> Self
    where
        Alice: Member<L, AliceMemberL>,
    {
        FanIn {
            phantom: PhantomData,
        }
    }
}

impl<L: LocationSet, QS: LocationSet, Alice: ChoreographyLocation, AliceMemberL>
    FanInChoreography<String> for FanIn<L, QS, Alice, AliceMemberL>
where
    Alice: Member<L, AliceMemberL>,
{
    type L = L;
    type QS = QS;
    type RS = LocationSet!(Alice);

    fn run<Q: ChoreographyLocation, QSSubsetL, RSSubsetL, QMemberL, QMemberQS>(
        &self,
        op: &impl ChoreoOp<Self::L>,
    ) -> MultiplyLocated<String, Self::RS>
    where
        Self::QS: Subset<Self::L, QSSubsetL>,
        Self::RS: Subset<Self::L, RSSubsetL>,
        Q: Member<Self::L, QMemberL>,
        Q: Member<Self::QS, QMemberQS>,
    {
        let msg_at_q = op.locally(Q::new(), |_| {
            format!("{} says hi to {}", Q::name(), Alice::name())
        });
        let msg_at_alice = op.comm(Q::new(), Alice::new(), &msg_at_q);
        return msg_at_alice;
    }
}

struct MainChoreography;
impl Choreography<Located<Quire<String, LocationSet!(Bob, Carol)>, Alice>> for MainChoreography {
    type L = LocationSet!(Alice, Bob, Carol);

    fn run(
        self,
        op: &impl ChoreoOp<Self::L>,
    ) -> Located<Quire<String, LocationSet!(Bob, Carol)>, Alice> {
        let v = op.fanin(<LocationSet!(Bob, Carol)>::new(), FanIn::new(Alice));
        op.locally(Alice, |un| {
            let m = un.unwrap(&v).get_map();
            println!(
                "Alice received: \"{}\" from Bob and \"{}\" from Carol",
                m.get(Bob::name()).unwrap_or(&String::from("ERROR")),
                m.get(Carol::name()).unwrap_or(&String::from("ERROR"))
            )
        });
        return v;
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
                alice_projector.epp_and_run(MainChoreography);
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Bob".to_string())
            .spawn(move || {
                bob_projector.epp_and_run(MainChoreography);
            })
            .unwrap(),
    );
    handles.push(
        thread::Builder::new()
            .name("Carol".to_string())
            .spawn(move || {
                carol_projector.epp_and_run(MainChoreography);
            })
            .unwrap(),
    );
    for handle in handles {
        handle.join().unwrap();
    }
}

#[cfg(test)]
mod tests {
    use chorus_lib::core::Runner;

    use super::*;

    #[test]
    fn test_projector() {
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
                    let quire_at_alice = alice_projector.epp_and_run(MainChoreography);
                    let m = alice_projector.unwrap(quire_at_alice).get_map();
                    assert_eq!(m.get(Bob::name()).unwrap(), "Bob says hi to Alice");
                    assert_eq!(m.get(Carol::name()).unwrap(), "Carol says hi to Alice");
                })
                .unwrap(),
        );
        handles.push(
            thread::Builder::new()
                .name("Bob".to_string())
                .spawn(move || {
                    bob_projector.epp_and_run(MainChoreography);
                })
                .unwrap(),
        );
        handles.push(
            thread::Builder::new()
                .name("Carol".to_string())
                .spawn(move || {
                    carol_projector.epp_and_run(MainChoreography);
                })
                .unwrap(),
        );
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_runner() {
        let runner = Runner::new();
        let quire_at_alice = runner.run(MainChoreography);
        let m = runner.unwrap(quire_at_alice).get_map();
        assert_eq!(m.get(Bob::name()).unwrap(), "Bob says hi to Alice");
        assert_eq!(m.get(Carol::name()).unwrap(), "Carol says hi to Alice");
    }
}
