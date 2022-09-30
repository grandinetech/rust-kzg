use blst_from_scratch::types::{fft_settings::FsFFTSettings, fr::FsFr, poly::FsPoly};
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<FsFr, FsFFTSettings, FsPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
