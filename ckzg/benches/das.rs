use ckzg::fftsettings::KzgFFTSettings;
use ckzg::finite::BlstFr;
use criterion::{criterion_group, criterion_main, Criterion};

fn bench_das_extension(c: &mut Criterion) {
    kzg_bench::benches::das::bench_das_extension::<BlstFr, KzgFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_das_extension
}

criterion_main!(benches);
