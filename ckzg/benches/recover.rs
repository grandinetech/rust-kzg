use ckzg::fftsettings::KzgFFTSettings;
use ckzg::finite::BlstFr;
use ckzg::poly::KzgPoly;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

fn bench_recover_(c: &mut Criterion) {
    bench_recover::<BlstFr, KzgFFTSettings, KzgPoly, KzgPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
