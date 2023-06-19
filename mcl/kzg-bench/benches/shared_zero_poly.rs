use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;
use rust_kzg_mcl::data_types::fr::Fr;
use rust_kzg_mcl::fk20_fft::FFTSettings;
use rust_kzg_mcl::kzg10::Polynomial;
use rust_kzg_mcl::mcl_methods::init;
use rust_kzg_mcl::CurveType;

fn bench_zero_poly_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_zero_poly::<Fr, FFTSettings, Polynomial>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
