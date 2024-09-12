use criterion::{criterion_group, criterion_main, Criterion};
use kzg::eip_4844::{
    blob_to_kzg_commitment_rust, bytes_to_blob, compute_blob_kzg_proof_rust,
    compute_cells_and_kzg_proofs_rust, compute_kzg_proof_rust, verify_blob_kzg_proof_batch_rust,
    verify_blob_kzg_proof_rust, verify_kzg_proof_rust,
};
use kzg_bench::benches::eip_4844::bench_eip_4844;
use rust_kzg_constantine::{
    eip_4844::load_trusted_setup_filename_rust,
    types::{
        fft_settings::CtFFTSettings,
        fp::CtFp,
        fr::CtFr,
        g1::{CtG1, CtG1Affine},
        g2::CtG2,
        kzg_settings::CtKZGSettings,
        poly::CtPoly,
    },
};

fn bench_eip_4844_(c: &mut Criterion) {
    bench_eip_4844::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFp, CtG1Affine>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &bytes_to_blob,
        &compute_kzg_proof_rust,
        &verify_kzg_proof_rust,
        &compute_blob_kzg_proof_rust,
        &compute_cells_and_kzg_proofs_rust,
        &verify_blob_kzg_proof_rust,
        &verify_blob_kzg_proof_batch_rust,
    );
}

criterion_group!(benches, bench_eip_4844_);
criterion_main!(benches);
