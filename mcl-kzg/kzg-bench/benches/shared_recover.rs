use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;
use mcl_rust::data_types::fr::Fr;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::kzg10::Polynomial;

use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn bench_recover_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_recover::<Fr, FFTSettings, Polynomial, Polynomial>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
