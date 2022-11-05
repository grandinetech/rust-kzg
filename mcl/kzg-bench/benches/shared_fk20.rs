use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::*;
use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::fk20_matrix::{FK20Matrix, FK20SingleMatrix};
use mcl_rust::kzg10::Polynomial;
use mcl_rust::kzg_settings::KZGSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn fk_single_da_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    fk_single_da::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20SingleMatrix>(
        c,
        &KZGSettings::generate_trusted_setup,
    )
}

fn fk_multi_da_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    fk_multi_da::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings, FK20Matrix>(
        c,
        &KZGSettings::generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = fk_single_da_, fk_multi_da_
}

criterion_main!(benches);
