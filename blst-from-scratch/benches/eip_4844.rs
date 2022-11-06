use blst_from_scratch::{types::{fr::FsFr, g1::FsG1, g2::FsG2, poly::FsPoly, fft_settings::FsFFTSettings, kzg_settings::FsKZGSettings}, eip_4844::{load_trusted_setup, blob_to_kzg_commitment, compute_aggregate_kzg_proof, verify_aggregate_kzg_proof}};
use criterion::{Criterion, criterion_group, criterion_main};
use kzg_bench::benches::eip_4844::{bench_compute_aggregate_kzg_proof, bench_verify_aggregate_kzg_proof};

fn bench_compute_aggregate_kzg_proof_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>
    (
        c,
        &load_trusted_setup,
        &compute_aggregate_kzg_proof,
    ) 
}

fn bench_verify_aggregate_kzg_proof_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>
    (
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
    ) 
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_compute_aggregate_kzg_proof_, bench_verify_aggregate_kzg_proof_
}

criterion_main!(benches);