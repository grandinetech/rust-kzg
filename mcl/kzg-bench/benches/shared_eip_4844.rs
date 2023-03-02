use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::eip_4844::{
    bench_compute_aggregate_kzg_proof, bench_verify_aggregate_kzg_proof,
};
use mcl_rust::eip_4844::*;
use mcl_rust::mcl_methods::init;
use mcl_rust::CurveType;

fn compute_aggregate_kzg_proof_bench_1(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_compute_aggregate_kzg_proof(c, &load_trusted_setup, &compute_aggregate_kzg_proof, 1)
}

fn compute_aggregate_kzg_proof_bench_2(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_compute_aggregate_kzg_proof(c, &load_trusted_setup, &compute_aggregate_kzg_proof, 2)
}

fn compute_aggregate_kzg_proof_bench_4(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_compute_aggregate_kzg_proof(c, &load_trusted_setup, &compute_aggregate_kzg_proof, 4)
}

fn compute_aggregate_kzg_proof_bench_8(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_compute_aggregate_kzg_proof(c, &load_trusted_setup, &compute_aggregate_kzg_proof, 8)
}

fn compute_aggregate_kzg_proof_bench_16(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_compute_aggregate_kzg_proof(c, &load_trusted_setup, &compute_aggregate_kzg_proof, 16)
}

fn verify_aggregate_kzg_proof_bench_1(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_verify_aggregate_kzg_proof(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
        1,
    )
}

fn verify_aggregate_kzg_proof_bench_2(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_verify_aggregate_kzg_proof(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
        2,
    )
}

fn verify_aggregate_kzg_proof_bench_4(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_verify_aggregate_kzg_proof(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
        4,
    )
}

fn verify_aggregate_kzg_proof_bench_8(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_verify_aggregate_kzg_proof(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
        8,
    )
}

fn verify_aggregate_kzg_proof_bench_16(c: &mut Criterion) {
    assert!(init(CurveType::BLS12_381));
    bench_verify_aggregate_kzg_proof(
        c,
        &load_trusted_setup,
        &blob_to_kzg_commitment,
        &compute_aggregate_kzg_proof,
        &verify_aggregate_kzg_proof,
        16,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = compute_aggregate_kzg_proof_bench_1, compute_aggregate_kzg_proof_bench_2, compute_aggregate_kzg_proof_bench_4, compute_aggregate_kzg_proof_bench_8, compute_aggregate_kzg_proof_bench_16, verify_aggregate_kzg_proof_bench_1, verify_aggregate_kzg_proof_bench_2, verify_aggregate_kzg_proof_bench_4, verify_aggregate_kzg_proof_bench_8, verify_aggregate_kzg_proof_bench_16
}

criterion_main!(benches);
