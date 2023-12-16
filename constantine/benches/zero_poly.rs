use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;
use rust_kzg_blst::types::{fft_settings::CtFFTSettings, fr::CtFr, poly::CtPoly};

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<CtFr, CtFFTSettings, CtPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
