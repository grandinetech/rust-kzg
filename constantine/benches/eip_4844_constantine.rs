// Same as eip_4844.rs, but using constantine implementations of verification functions

use criterion::{criterion_group, criterion_main, Criterion};
use kzg::eip_4844::{
    blob_to_kzg_commitment_rust, bytes_to_blob, compute_blob_kzg_proof_rust,
    compute_kzg_proof_rust, verify_blob_kzg_proof_batch_rust, verify_blob_kzg_proof_rust,
    verify_kzg_proof_rust,
};
use kzg_bench::benches::eip_4844::bench_eip_4844;
use rust_kzg_constantine::{
    eip_4844::load_trusted_setup_filename_rust,
    mixed_kzg_settings::mixed_eip_4844::load_trusted_setup_filename_mixed,
    mixed_kzg_settings::{
        mixed_eip_4844::{
            blob_to_kzg_commitment_mixed, compute_blob_kzg_proof_mixed, compute_kzg_proof_mixed,
            verify_blob_kzg_proof_batch_mixed, verify_blob_kzg_proof_mixed, verify_kzg_proof_mixed,
        },
        mixed_kzg_settings::MixedKzgSettings,
    },
    types::{
        fft_settings::CtFFTSettings, fr::CtFr, g1::CtG1, g2::CtG2, kzg_settings::CtKZGSettings,
        poly::CtPoly,
    },
};

fn bench_eip_4844_constantine_(c: &mut Criterion) {
    // Mixed KZG eip_4844 test - lots of conversions so not indicative of 'true' performance
    bench_eip_4844::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, MixedKzgSettings>(
        c,
        &load_trusted_setup_filename_mixed,
        &blob_to_kzg_commitment_mixed,
        &bytes_to_blob,
        &compute_kzg_proof_mixed,
        &verify_kzg_proof_mixed,
        &compute_blob_kzg_proof_mixed,
        &verify_blob_kzg_proof_mixed,
        &verify_blob_kzg_proof_batch_mixed,
    );
}

criterion_group!(benches, bench_eip_4844_constantine_);
criterion_main!(benches);
