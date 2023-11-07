use chorus_lib::core::{
    ChoreoOp, Choreography, ChoreographyLocation, LocationSet, Projector, Transport,
};
use chorus_lib::transport::local::{LocalTransport, LocalTransportChannelBuilder};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::thread::spawn;

#[derive(ChoreographyLocation)]
struct Alice;

#[derive(ChoreographyLocation)]
struct Bob;

struct CommChoreography {
    n: u64,
}

impl Choreography for CommChoreography {
    type L = LocationSet!(Alice, Bob);

    fn run(self, op: &impl ChoreoOp<Self::L>) {
        for _ in 0..self.n {
            op.comm(Bob, Alice, &op.locally::<f32, _, _>(Bob, |_| 1.0));
        }
    }
}

fn comm_choreography(n: u64) {
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
            let c = CommChoreography { n };
            projector.epp_and_run(c);
        }));
    }
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let transport = LocalTransport::new(Bob, channel);
            let projector = Projector::new(Bob, transport);
            let c = CommChoreography { n };
            projector.epp_and_run(c);
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn comm_handwritten_alice(n: u64, transport: &LocalTransport<LocationSet!(Bob, Alice), Alice>) {
    for _ in 0..n {
        transport.receive::<f32>(Bob::name(), Alice::name());
    }
}

fn comm_handwritten_bob(n: u64, transport: &LocalTransport<LocationSet!(Bob, Alice), Bob>) {
    for _ in 0..n {
        transport.send::<f32>(Bob::name(), Alice::name(), &1.0);
    }
}

fn comm_handwritten(n: u64) {
    let channel = LocalTransportChannelBuilder::new()
        .with(Alice)
        .with(Bob)
        .build();
    let mut handles = Vec::new();
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let transport = LocalTransport::new(Alice, channel);
            comm_handwritten_alice(n, &transport);
        }));
    }
    {
        let channel = channel.clone();
        handles.push(spawn(move || {
            let transport = LocalTransport::new(Bob, channel);
            comm_handwritten_bob(n, &transport);
        }));
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn bench_comm(c: &mut Criterion) {
    let mut group = c.benchmark_group("Comm");
    let range = [2000, 4000, 6000, 8000, 10000];
    for i in range.iter() {
        group.bench_with_input(BenchmarkId::new("Handwritten", i), i, |b, i| {
            b.iter(|| comm_handwritten(black_box(*i)))
        });
    }
    for i in range.iter() {
        group.bench_with_input(BenchmarkId::new("Choreographic", i), i, |b, i| {
            b.iter(|| comm_choreography(black_box(*i)))
        });
    }
    group.finish();
}

criterion_group!(benches, bench_comm);
criterion_main!(benches);
