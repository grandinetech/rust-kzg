use criterion::{criterion_group, criterion_main, Criterion};
use ckzg::fftsettings::KzgFFTSettings;
use ckzg::finite::BlstFr;

fn bench_das_extension(c: &mut Criterion) {
    kzg_bench::benches::das::bench_das_extension::<BlstFr, KzgFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = bench_das_extension
}

criterion_main!(benches);