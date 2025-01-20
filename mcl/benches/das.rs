use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;
use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
use rust_kzg_mcl::types::fr::MclFr;

fn bench_das_extension_(c: &mut Criterion) {
    bench_das_extension::<MclFr, MclFFTSettings>(c)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_das_extension_
}

criterion_main!(benches);
