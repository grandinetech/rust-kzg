use mcl_rust::data_types::{fr::Fr, g1::G1, g2::G2};
use mcl_rust::eip_4844::*;
use mcl_rust::fk20_fft::FFTSettings;
use mcl_rust::kzg10::Polynomial;
use mcl_rust::kzg_settings::KZGSettings;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::eip_4844::bench_eip_4844;

fn bench_eip_4844_(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_eip_4844::<Fr, G1, G2, Polynomial, FFTSettings, KZGSettings>(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &bytes_to_bls_field,
        &compute_kzg_proof,
        &compute_blob_kzg_proof,
        &verify_blob_kzg_proof,
        &verify_blob_kzg_proof_batch,
    );
}

criterion_group!(benches, bench_eip_4844_,);
criterion_main!(benches);
