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
            i = op.broadcast(Alice, op.locally(Alice, |_| i + 1));
            if i >= self.n {
                break;
            }
            i = op.broadcast(Bob, op.locally(Bob, |_| i + 1));
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
    let mut handles = Vec::new();
    let channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .build();
    {
        let channel = channel.clone();
        let transport = LocalTransport::new(Alice, channel);
        handles.push(spawn(move || {
            let mut i = 0;
            loop {
                i += 1;
                transport.send(Alice::name(), Bob::name(), &i);
                if i >= n {
                    break;
                }
                i = transport.receive(Bob::name(), Alice::name());
                if i >= n {
                    break;
                }
            }
        }));
    }
    {
        let channel = channel.clone();
        let transport = LocalTransport::new(Bob, channel);
        handles.push(spawn(move || {
            let mut i = 0;
            loop {
                i = transport.receive(Alice::name(), Bob::name());
                if i >= n {
                    break;
                }
                i += 1;
                transport.send(Bob::name(), Alice::name(), &i);
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
    for i in [10000].iter() {
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
