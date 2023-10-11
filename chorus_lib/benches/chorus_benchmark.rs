extern crate chorus_lib;

use std::thread::spawn;

use chorus_lib::{
    core::{ChoreoOp, Choreography, ChoreographyLocation, LocationSet, Projector, Transport},
    transport::local::{LocalTransport, LocalTransportChannelBuilder},
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

struct AddToNChoreography {
    n: u64,
}

impl Choreography for AddToNChoreography {
    type L = LocationSet!(Alice, Bob);

    fn run(self, op: &impl ChoreoOp<Self::L>) -> () {
        let mut i = 0;
        loop {
            let tmp = op.locally(Alice, |_| i + 1);
            i = op.broadcast(Alice, tmp);
            if i >= self.n {
                break;
            }
            let tmp = op.locally(Bob, |_| i + 1);
            i = op.broadcast(Bob, tmp);
            if i >= self.n {
                break;
            }
        }
    }
}

fn add_to_n_choreography(n: u64) {
    let c1 = AddToNChoreography { n };
    let c2 = AddToNChoreography { n };
    let channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .build();
    let mut handles = Vec::new();
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let transport = LocalTransport::new(Alice, channel);
            let projector = Projector::new(Alice, transport);
            projector.epp_and_run(c1);
        }));
    }
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let transport = LocalTransport::new(Bob, channel);
            let projector = Projector::new(Bob, transport);
            projector.epp_and_run(c2);
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn add_to_n(n: u64) {
    let n = n;
    let mut handles = Vec::new();
    let channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .build();
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let n = n;
            let transport = LocalTransport::new(Alice, channel);
            let mut i: u64 = 0;
            loop {
                i += 1;
                transport.send("Alice", "Bob", &i);
                if i >= n {
                    break;
                }
                i = transport.receive("Bob", "Alice");
                if i >= n {
                    break;
                }
            }
        }));
    }
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let n = n;
            let transport = LocalTransport::new(Bob, channel);
            loop {
                let mut i = transport.receive::<u64>("Alice", "Bob");
                if i >= n {
                    break;
                }
                i += 1;
                transport.send("Bob", "Alice", &i);
                if i >= n {
                    break;
                }
            }
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn bench_add_to_n_local(c: &mut Criterion) {
    let mut group = c.benchmark_group("Add to N (Local Transport)");
    for i in [1000].iter() {
        group.bench_with_input(BenchmarkId::new("Choreographic", i), i, |b, i| {
            b.iter(|| add_to_n_choreography(*i))
        });
        group.bench_with_input(BenchmarkId::new("Handwritten", i), i, |b, i| {
            b.iter(|| add_to_n(*i))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_add_to_n_local);
criterion_main!(benches);
