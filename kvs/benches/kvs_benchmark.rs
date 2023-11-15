use std::thread::spawn;

use chorus_lib::{
    core::Projector,
    transport::http::{HttpTransport, HttpTransportConfigBuilder},
};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kvs::handwritten::{backup, primary};
use kvs::shared::*;
use kvs::{choreographic::*, handwritten::client};
use rand::Rng;

const KEY_MAX: u64 = 100;

pub fn random_request<R: Rng>(rng: &mut R, read_write_ratio: f64) -> kvs::shared::Request {
    let key = rng.gen_range(0..KEY_MAX).to_string();
    if rng.gen_bool(read_write_ratio) {
        Request::Get(key)
    } else {
        let value = rng.gen::<u64>().to_string();
        Request::Put(key, value)
    }
}

fn kvs_choreographic(num_requests: u64) {
    let mut handles = Vec::new();
    handles.push(spawn(move || {
        let config = HttpTransportConfigBuilder::for_target(Client, ("0.0.0.0", 8080))
            .with(Primary, ("0.0.0.0", 8090))
            .with(Backup, ("0.0.0.0", 8100))
            .build();
        let transport = HttpTransport::new(config);
        let projector = Projector::new(Client, transport);
        let mut r = rand::thread_rng();
        for _ in 0..num_requests {
            let request = random_request(&mut r, 0.5);
            let choreography = PrimaryBackupKvsChoreography {
                state: (projector.remote(Primary), projector.remote(Backup)),
                request: projector.local(request),
            };
            projector.epp_and_run(choreography);
        }
    }));
    handles.push(spawn(move || {
        let config = HttpTransportConfigBuilder::for_target(Primary, ("0.0.0.0", 8090))
            .with(Client, ("0.0.0.0", 8080))
            .with(Backup, ("0.0.0.0", 8100))
            .build();
        let transport = HttpTransport::new(config);
        let projector = Projector::new(Primary, transport);
        let state = State::default();
        for _ in 0..num_requests {
            let choreography = PrimaryBackupKvsChoreography {
                state: (projector.local(&state), projector.remote(Backup)),
                request: projector.remote(Client),
            };
            projector.epp_and_run(choreography);
        }
    }));
    handles.push(spawn(move || {
        let config = HttpTransportConfigBuilder::for_target(Backup, ("0.0.0.0", 8100))
            .with(Primary, ("0.0.0.0", 8090))
            .with(Client, ("0.0.0.0", 8080))
            .build();
        let transport = HttpTransport::new(config);
        let projector = Projector::new(Backup, transport);
        let state = State::default();
        for _ in 0..num_requests {
            let choreography = PrimaryBackupKvsChoreography {
                state: (projector.remote(Primary), projector.local(&state)),
                request: projector.remote(Client),
            };
            projector.epp_and_run(choreography);
        }
    }));
    for handle in handles {
        handle.join().unwrap();
    }
}

fn kvs_handwritten(num_requests: u64) {
    let mut handles = Vec::new();
    handles.push(spawn(move || {
        let config = HttpTransportConfigBuilder::for_target(Client, ("0.0.0.0", 8080))
            .with(Primary, ("0.0.0.0", 8090))
            .with(Backup, ("0.0.0.0", 8100))
            .build();
        let transport = HttpTransport::new(config);
        let mut r = rand::thread_rng();
        for _ in 0..num_requests {
            let request = random_request(&mut r, 0.5);
            client(&transport, request);
        }
    }));
    handles.push(spawn(move || {
        let config = HttpTransportConfigBuilder::for_target(Primary, ("0.0.0.0", 8090))
            .with(Client, ("0.0.0.0", 8080))
            .with(Backup, ("0.0.0.0", 8100))
            .build();
        let transport = HttpTransport::new(config);
        let state = State::default();
        for _ in 0..num_requests {
            primary(&transport, &state);
        }
    }));
    handles.push(spawn(move || {
        let config = HttpTransportConfigBuilder::for_target(Backup, ("0.0.0.0", 8100))
            .with(Primary, ("0.0.0.0", 8090))
            .with(Client, ("0.0.0.0", 8080))
            .build();
        let transport = HttpTransport::new(config);
        let state = State::default();
        for _ in 0..num_requests {
            backup(&transport, &state);
        }
    }));
    for handle in handles {
        handle.join().unwrap();
    }
}

fn kvs_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("kvs");
    group.bench_function("choreographic", |b| {
        b.iter(|| kvs_choreographic(black_box(100)))
    });
    group.bench_function("handwritten", |b| {
        b.iter(|| kvs_handwritten(black_box(100)))
    });
}

criterion_group!(benches, kvs_benchmark);
criterion_main!(benches);
