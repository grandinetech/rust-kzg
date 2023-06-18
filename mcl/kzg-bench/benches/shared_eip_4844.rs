use rust_kzg_mcl::data_types::{fr::Fr, g1::G1, g2::G2};
use rust_kzg_mcl::eip_4844::*;
use rust_kzg_mcl::fk20_fft::FFTSettings;
use rust_kzg_mcl::kzg10::Polynomial;
use rust_kzg_mcl::kzg_settings::KZGSettings;
use rust_kzg_mcl::mcl_methods::init;
use rust_kzg_mcl::CurveType;

use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::eip_4844::bench_eip_4844;

fn bench_eip_4844_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_eip_4844::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &bytes_to_blob,
        &compute_kzg_proof,
        &verify_kzg_proof,
        &compute_blob_kzg_proof,
        &verify_blob_kzg_proof,
        &verify_blob_kzg_proof_batch,
    );
}

criterion_group!(benches, bench_eip_4844_,);
criterion_main!(benches);
