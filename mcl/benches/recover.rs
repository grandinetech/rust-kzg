use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;
use rust_kzg_mcl::types::{fft_settings::MclFFTSettings, fr::MclFr, poly::MclPoly};

pub fn bench_recover_(c: &mut Criterion) {
    bench_recover::<MclFr, MclFFTSettings, MclPoly, MclPoly>(c)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
