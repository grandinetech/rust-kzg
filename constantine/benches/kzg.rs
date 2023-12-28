use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{bench_commit_to_poly, bench_compute_proof_single};
use rust_kzg_constantine::types::fft_settings::CtFFTSettings;
use rust_kzg_constantine::types::fr::CtFr;
use rust_kzg_constantine::types::g1::CtG1;
use rust_kzg_constantine::types::g2::CtG2;
use rust_kzg_constantine::types::kzg_settings::CtKZGSettings;
use rust_kzg_constantine::types::poly::CtPoly;
use rust_kzg_constantine::utils::generate_trusted_setup;

fn bench_commit_to_poly_(c: &mut Criterion) {
    bench_commit_to_poly::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings>(
        c,
        &generate_trusted_setup,
    )
}

fn bench_compute_proof_single_(c: &mut Criterion) {
    bench_compute_proof_single::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings>(
        c,
        &generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_commit_to_poly_, bench_compute_proof_single_
}

criterion_main!(benches);
