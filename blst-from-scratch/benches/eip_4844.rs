use blst_from_scratch::{
    eip_4844::{
        blob_to_kzg_commitment_rust, compute_aggregate_kzg_proof_rust, load_trusted_setup_filename_rust,
        verify_aggregate_kzg_proof_rust,
    },
    types::{
        fft_settings::FsFFTSettings, fr::FsFr, g1::FsG1, g2::FsG2, kzg_settings::FsKZGSettings,
        poly::FsPoly,
    },
};
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::eip_4844::{
    bench_compute_aggregate_kzg_proof, bench_verify_aggregate_kzg_proof,
};

fn bench_compute_aggregate_kzg_proof_1_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &compute_aggregate_kzg_proof_rust,
        1,
    )
}

fn bench_compute_aggregate_kzg_proof_2_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &compute_aggregate_kzg_proof_rust,
        2,
    )
}
fn bench_compute_aggregate_kzg_proof_4_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &compute_aggregate_kzg_proof_rust,
        4,
    )
}

fn bench_compute_aggregate_kzg_proof_8_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &compute_aggregate_kzg_proof_rust,
        8,
    )
}

fn bench_compute_aggregate_kzg_proof_16_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &compute_aggregate_kzg_proof_rust,
        16,
    )
}

fn bench_verify_aggregate_kzg_proof_1_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &compute_aggregate_kzg_proof_rust,
        &verify_aggregate_kzg_proof_rust,
        1,
    )
}
fn bench_verify_aggregate_kzg_proof_2_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &compute_aggregate_kzg_proof_rust,
        &verify_aggregate_kzg_proof_rust,
        2,
    )
}
fn bench_verify_aggregate_kzg_proof_4_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &compute_aggregate_kzg_proof_rust,
        &verify_aggregate_kzg_proof_rust,
        4,
    )
}
fn bench_verify_aggregate_kzg_proof_8_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &compute_aggregate_kzg_proof_rust,
        &verify_aggregate_kzg_proof_rust,
        8,
    )
}
fn bench_verify_aggregate_kzg_proof_16_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup_filename_rust,
        &blob_to_kzg_commitment_rust,
        &compute_aggregate_kzg_proof_rust,
        &verify_aggregate_kzg_proof_rust,
        16,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets =
        bench_compute_aggregate_kzg_proof_1_, bench_compute_aggregate_kzg_proof_2_,
        bench_compute_aggregate_kzg_proof_4_, bench_compute_aggregate_kzg_proof_8_, bench_compute_aggregate_kzg_proof_16_,
        bench_verify_aggregate_kzg_proof_1_, bench_verify_aggregate_kzg_proof_2_,
        bench_verify_aggregate_kzg_proof_4_, bench_verify_aggregate_kzg_proof_8_, bench_verify_aggregate_kzg_proof_16_
}

criterion_main!(benches);
