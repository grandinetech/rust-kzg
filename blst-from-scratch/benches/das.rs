use blst_from_scratch::types::fft_settings::FsFFTSettings;
use blst_from_scratch::types::fr::FsFr;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;

fn bench_das_extension_(c: &mut Criterion) {
    bench_das_extension::<FsFr, FsFFTSettings>(c)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_das_extension_
}

criterion_main!(benches);
