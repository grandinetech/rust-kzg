use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use mcl_rust::data_types::fr::Fr;
use mcl_rust::data_types::g1::G1;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn bench_fft_fr_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_fft_fr::<Fr, FFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_fft_g1::<Fr, G1, FFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
