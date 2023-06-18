use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::*;
use rust_kzg_mcl::data_types::{fr::Fr, g1::G1, g2::G2};
use rust_kzg_mcl::fk20_fft::FFTSettings;
use rust_kzg_mcl::fk20_matrix::{FK20Matrix, FK20SingleMatrix};
use rust_kzg_mcl::kzg10::Polynomial;
use rust_kzg_mcl::kzg_settings::KZGSettings;
use rust_kzg_mcl::mcl_methods::init;
use rust_kzg_mcl::CurveType;

fn bench_fk_single_da_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_fk_single_da::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20SingleMatrix>(
        c,
        &KZGSettings::generate_trusted_setup,
    )
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_fk_multi_da::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(
        c,
        &KZGSettings::generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
