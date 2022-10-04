use ckzg::fftsettings::KzgFFTSettings;
use ckzg::finite::BlstFr;
use ckzg::poly::KzgPoly;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<BlstFr, KzgFFTSettings, KzgPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
