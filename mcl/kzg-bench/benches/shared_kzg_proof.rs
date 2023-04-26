use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{bench_commit_to_poly, bench_compute_proof_single};
use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::kzg10::Polynomial;
use mcl_rust::kzg_settings::KZGSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

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
