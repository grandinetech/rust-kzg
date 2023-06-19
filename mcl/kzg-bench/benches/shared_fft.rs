use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use rust_kzg_mcl::data_types::fr::Fr;
use rust_kzg_mcl::data_types::g1::G1;
use rust_kzg_mcl::fk20_fft::FFTSettings;
use rust_kzg_mcl::mcl_methods::init;
use rust_kzg_mcl::CurveType;

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
