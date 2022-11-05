use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

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

// pub fn bench_das_extension<TFr: Fr, TFFTSettings: FFTSettings<TFr> + DAS<TFr>>(c: &mut Criterion) {
