use blst_from_scratch::types::fft_settings::FsFFTSettings;
use blst_from_scratch::types::fr::FsFr;
use blst_from_scratch::types::g1::FsG1;
use blst_from_scratch::types::g2::FsG2;
use blst_from_scratch::types::kzg_settings::FsKZGSettings;
use blst_from_scratch::types::poly::FsPoly;
use blst_from_scratch::utils::generate_trusted_setup;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{commit_to_poly, compute_proof_single};

fn commit_to_poly_(c: &mut Criterion) {
    commit_to_poly::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &generate_trusted_setup,
    )
}

fn compute_proof_single_(c: &mut Criterion) {
    compute_proof_single::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = commit_to_poly_, compute_proof_single_
}

criterion_main!(benches);
