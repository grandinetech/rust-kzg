use arkworks::eip_4844::{
    blob_to_kzg_commitment, bytes_to_blob, compute_blob_kzg_proof, compute_kzg_proof,
    load_trusted_setup, verify_blob_kzg_proof, verify_blob_kzg_proof_batch, verify_kzg_proof,
};
use arkworks::kzg_proofs::{FFTSettings, KZGSettings};
use arkworks::kzg_types::{ArkG1, ArkG2, FsFr};
use arkworks::utils::PolyData;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::eip_4844::bench_eip_4844;

fn bench_eip_4844_(c: &mut Criterion) {
    bench_eip_4844::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
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
