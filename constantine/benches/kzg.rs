use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{bench_commit_to_poly, bench_compute_proof_single};
use rust_kzg_constantine::eip_7594::CtBackend;
use rust_kzg_constantine::utils::generate_trusted_setup;

fn bench_commit_to_poly_(c: &mut Criterion) {
    bench_commit_to_poly::<CtBackend>(c, &generate_trusted_setup)
}

fn bench_compute_proof_single_(c: &mut Criterion) {
    bench_compute_proof_single::<CtBackend>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_commit_to_poly_, bench_compute_proof_single_
}

criterion_main!(benches);
