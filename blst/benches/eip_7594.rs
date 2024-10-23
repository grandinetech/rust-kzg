use criterion::{criterion_group, criterion_main, Criterion};
use kzg::{
    eip_4844::{blob_to_kzg_commitment_rust, bytes_to_blob},
    eip_7594::{
        compute_cells_and_kzg_proofs, recover_cells_and_kzg_proofs, verify_cell_kzg_proof_batch,
    },
};
use kzg_bench::benches::eip_7594::bench_eip_7594;
use rust_kzg_blst::{
    eip_4844::load_trusted_setup_filename_rust,
    types::{
        fft_settings::FsFFTSettings,
        fp::FsFp,
        fr::FsFr,
        g1::{FsG1, FsG1Affine},
        g2::FsG2,
        kzg_settings::FsKZGSettings,
        poly::FsPoly,
    },
};

fn bench_eip_7594_(c: &mut Criterion) {
    bench_eip_7594::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFp, FsG1Affine>(
        c,
        &load_trusted_setup_filename_rust,
        &bytes_to_blob,
        &blob_to_kzg_commitment_rust,
        &compute_cells_and_kzg_proofs,
        &recover_cells_and_kzg_proofs,
        &verify_cell_kzg_proof_batch,
    );
}

criterion_group!(benches, bench_eip_7594_);
criterion_main!(benches);
