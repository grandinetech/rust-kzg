use criterion::{criterion_group, criterion_main, Criterion};
use kzg::eip_4844::{
    blob_to_kzg_commitment_rust, bytes_to_blob, compute_blob_kzg_proof_rust,
    compute_kzg_proof_rust, verify_blob_kzg_proof_batch_rust, verify_blob_kzg_proof_rust,
    verify_kzg_proof_rust,
};
use kzg_bench::benches::eip_4844::bench_eip_4844;
use rust_kzg_arkworks5::eip_4844::load_trusted_setup_filename_rust;
use rust_kzg_arkworks5::kzg_proofs::{FFTSettings, KZGSettings};
use rust_kzg_arkworks5::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG2};
use rust_kzg_arkworks5::utils::PolyData;

fn bench_eip_4844_(c: &mut Criterion) {
    bench_eip_4844::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, ArkFp, ArkG1Affine>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &bytes_to_blob,
        &compute_kzg_proof_rust,
        &verify_kzg_proof_rust,
        &compute_blob_kzg_proof_rust,
        &verify_blob_kzg_proof_rust,
        &verify_blob_kzg_proof_batch_rust,
    );
}

criterion_group!(benches, bench_eip_4844_,);
criterion_main!(benches);
