use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::kzg_proof;
use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::kzg10::Polynomial;
use mcl_rust::kzg_settings::KZGSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn kzg_proof_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    kzg_proof::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
        c,
        &KZGSettings::generate_trusted_setup,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = kzg_proof_
}

criterion_main!(benches);
