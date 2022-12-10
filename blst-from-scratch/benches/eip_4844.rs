use blst_from_scratch::{
    eip_4844::{
        blob_to_kzg_commitment, compute_aggregate_kzg_proof, load_trusted_setup,
        verify_aggregate_kzg_proof,
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

fn bench_compute_aggregate_kzg_proof_2_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup,
        &compute_aggregate_kzg_proof,

        2,
    )
}


fn bench_compute_aggregate_kzg_proof_4_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup,
        &compute_aggregate_kzg_proof,

        4,
    )
}

fn bench_compute_aggregate_kzg_proof_8_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup,
        &compute_aggregate_kzg_proof,
        8,
    )
}

fn bench_compute_aggregate_kzg_proof_16_(c: &mut Criterion) {
    bench_compute_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
        c,
        &load_trusted_setup,
        &compute_aggregate_kzg_proof,
        16,
    )
}

fn bench_verify_aggregate_kzg_proof_(c: &mut Criterion) {
    bench_verify_aggregate_kzg_proof::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings>(
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
    targets = bench_compute_aggregate_kzg_proof_2_, bench_compute_aggregate_kzg_proof_4_, bench_compute_aggregate_kzg_proof_8_, bench_compute_aggregate_kzg_proof_16_, bench_verify_aggregate_kzg_proof_
}

criterion_main!(benches);
