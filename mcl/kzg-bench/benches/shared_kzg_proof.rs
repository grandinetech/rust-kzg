use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{bench_commit_to_poly, bench_compute_proof_single};
use rust_kzg_mcl::data_types::{fr::Fr, g1::G1, g2::G2};
use rust_kzg_mcl::fk20_fft::FFTSettings;
use rust_kzg_mcl::kzg10::Polynomial;
use rust_kzg_mcl::kzg_settings::KZGSettings;
use rust_kzg_mcl::mcl_methods::init;
use rust_kzg_mcl::CurveType;

fn bench_commit_to_poly_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_commit_to_poly::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
        c,
        &KZGSettings::generate_trusted_setup,
    );
}

fn bench_compute_proof_single_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_compute_proof_single::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
        c,
        &KZGSettings::generate_trusted_setup,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_commit_to_poly_, bench_compute_proof_single_
}

criterion_main!(benches);
