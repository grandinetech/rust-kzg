use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::kzg10::Polynomial;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

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
