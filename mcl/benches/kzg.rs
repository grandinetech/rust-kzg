use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{bench_commit_to_poly, bench_compute_proof_single};
use rust_kzg_mcl::types::fft_settings::FsFFTSettings;
use rust_kzg_mcl::types::fp::FsFp;
use rust_kzg_mcl::types::fr::FsFr;
use rust_kzg_mcl::types::g1::{FsG1, FsG1Affine};
use rust_kzg_mcl::types::g2::FsG2;
use rust_kzg_mcl::types::kzg_settings::FsKZGSettings;
use rust_kzg_mcl::types::poly::FsPoly;
use rust_kzg_mcl::utils::generate_trusted_setup;

fn bench_commit_to_poly_(c: &mut Criterion) {
    bench_commit_to_poly::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFp, FsG1Affine>(
        c,
        &generate_trusted_setup,
    )
}

fn bench_compute_proof_single_(c: &mut Criterion) {
    bench_compute_proof_single::<
        FsFr,
        FsG1,
        FsG2,
        FsPoly,
        FsFFTSettings,
        FsKZGSettings,
        FsFp,
        FsG1Affine,
    >(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_commit_to_poly_, bench_compute_proof_single_
}

criterion_main!(benches);
