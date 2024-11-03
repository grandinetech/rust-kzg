use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;
use rust_kzg_mcl::data_types::fr::Fr;
use rust_kzg_mcl::fk20_fft::FFTSettings;
use rust_kzg_mcl::mcl_methods::init;
use rust_kzg_mcl::CurveType;

fn bench_das_extension_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_das_extension::<Fr, FFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_das_extension_
}

criterion_main!(benches);
