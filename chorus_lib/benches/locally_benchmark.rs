use chorus_lib::core::{ChoreoOp, Choreography, ChoreographyLocation, Located, LocationSet};
use criterion::{
    black_box, criterion_group, criterion_main, AxisScale, BenchmarkId, Criterion,
    PlotConfiguration,
};

#[derive(ChoreographyLocation)]
struct Alice;

struct AddToNLocallyChoreography {
    n: f64,
}
impl Choreography<Located<f64, Alice>> for AddToNLocallyChoreography {
    type L = LocationSet!(Alice);

    fn run(self, op: &impl ChoreoOp<Self::L>) -> Located<f64, Alice> {
        let mut i = op.locally(Alice, |_| 0.0);
        for _ in 0..self.n.ceil() as u64 {
            i = op.locally(Alice, |un| *un.unwrap(&i) + 1.0);
        }
        return i;
    }
}

fn add_to_n_locally_choreographic(n: f64) {
    let c = AddToNLocallyChoreography { n };
    let channel = chorus_lib::transport::local::LocalTransportChannelBuilder::new()
        .with(Alice)
        .build();
    let transport = chorus_lib::transport::local::LocalTransport::new(Alice, channel);
    let projector = chorus_lib::core::Projector::new(Alice, transport);
    let result = projector.epp_and_run(c);
    assert_eq!(projector.unwrap(result), n);
}

fn add_to_n_locally_handwritten(n: f64) {
    let mut a: f64 = 0.0;
    for _ in 0..n.ceil() as u64 {
        a += 1.0;
    }
    assert_eq!(a, n);
}

fn bench_add_to_n_locally(c: &mut Criterion) {
    let mut group = c.benchmark_group("Locally");
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    // group.plot_config(plot_config);
    // let range = [1, 10, 100, 1000, 10000];
    let range = [2000, 4000, 6000, 8000, 10000];
    for i in range.iter() {
        group.bench_with_input(BenchmarkId::new("Handwritten", i), i, |b, i| {
            b.iter(|| add_to_n_locally_handwritten(black_box(*i as f64)))
        });
    }
    for i in range.iter() {
        group.bench_with_input(BenchmarkId::new("Choreographic", i), i, |b, i| {
            b.iter(|| add_to_n_locally_choreographic(black_box(*i as f64)))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_add_to_n_locally);
criterion_main!(benches);
