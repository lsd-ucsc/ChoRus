use chorus_lib::{
    core::{
        ChoreoOp, Choreography, ChoreographyLocation, Located, LocationSet, MultiplyLocated,
        Projector, Superposition,
    },
    transport::local::{LocalTransport, LocalTransportChannelBuilder},
};
use rand::Rng;
use serde::{Deserialize, Serialize};

type Query = String;

#[derive(Serialize, Deserialize)]
enum Choice {
    Alice,
    Bob,
}

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

#[derive(ChoreographyLocation)]
struct Carol;

struct MainChoreography;

impl Choreography for MainChoreography {
    type L = LocationSet!(Alice, Bob, Carol);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let choice = op.locally(Alice, |_| {
            let mut rng = rand::thread_rng();
            let choice: bool = rng.gen();
            if choice {
                Choice::Alice
            } else {
                Choice::Bob
            }
        });
        let ChoiceAndQuery(choice, query_at_alice) = op.enclave(ChooseQueryChoreography {
            alice_choice: choice,
        });
        let query_at_carol = op.comm(Alice, Carol, &query_at_alice);
        let response_at_carol = op.locally(Carol, |un| {
            let query = un.unwrap(&query_at_carol);
            println!("Carol received query: {}", query);
            let r = format!("Carol's response to {}", query);
            return r;
        });
        let response = op.broadcast(Carol, response_at_carol);
        op.enclave(TerminalChoreography { choice, response });
    }
}

#[derive(Superposition)]
struct ChoiceAndQuery(
    MultiplyLocated<Choice, LocationSet!(Alice, Bob)>,
    Located<Query, Alice>,
);

struct ChooseQueryChoreography {
    alice_choice: Located<Choice, Alice>,
}

impl Choreography<ChoiceAndQuery> for ChooseQueryChoreography {
    type L = LocationSet!(Alice, Bob);
    fn run(self, op: &impl ChoreoOp<Self::L>) -> ChoiceAndQuery {
        let choice = op.broadcast(Alice, self.alice_choice);
        let query = match choice {
            Choice::Alice => op.locally(Alice, |_| "Alice's query".to_string()),
            Choice::Bob => {
                let bob_query = op.locally(Bob, |_| "Bob's query".to_string());
                op.comm(Bob, Alice, &bob_query)
            }
        };
        return ChoiceAndQuery(op.unnaked(choice), query);
    }
}

struct TerminalChoreography {
    choice: MultiplyLocated<Choice, LocationSet!(Alice, Bob)>,
    response: String,
}

impl Choreography for TerminalChoreography {
    type L = LocationSet!(Alice, Bob);
    fn run(self, op: &impl ChoreoOp<Self::L>) {
        let choice = op.naked(self.choice);
        match choice {
            Choice::Alice => {
                op.locally(Alice, |_| {
                    println!("Alice received response: {}", self.response);
                });
            }
            Choice::Bob => {
                op.locally(Bob, |_| {
                    println!("Bob received response: {}", self.response);
                });
            }
        }
    }
}

fn main() {
    let mut handles = Vec::new();
    let transport_channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .with(Carol)
        .build();
    {
        let transport = LocalTransport::new(Alice, transport_channel.clone());
        handles.push(std::thread::spawn(move || {
            let p = Projector::new(Alice, transport);
            p.epp_and_run(MainChoreography);
        }));
    }
    {
        let transport = LocalTransport::new(Bob, transport_channel.clone());
        handles.push(std::thread::spawn(move || {
            let p = Projector::new(Bob, transport);
            p.epp_and_run(MainChoreography);
        }));
    }
    {
        let transport = LocalTransport::new(Carol, transport_channel.clone());
        handles.push(std::thread::spawn(move || {
            let p = Projector::new(Carol, transport);
            p.epp_and_run(MainChoreography);
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}
