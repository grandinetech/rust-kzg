use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::eip_4844::bench_eip_4844;
use zkcrypto::eip_4844::{
    blob_to_kzg_commitment, bytes_to_bls_field, compute_blob_kzg_proof, compute_kzg_proof,
    load_trusted_setup, verify_blob_kzg_proof, verify_blob_kzg_proof_batch,
};
use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::kzg_proofs::KZGSettings;
use zkcrypto::kzg_types::ZkG2Projective;
use zkcrypto::poly::KzgPoly;
use zkcrypto::utils::ZkG1Projective;
use zkcrypto::zkfr::blsScalar;

fn bench_eip_4844_(c: &mut Criterion) {
    bench_eip_4844::<blsScalar, ZkG1Projective, ZkG2Projective, KzgPoly, ZkFFTSettings, KZGSettings>(
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
