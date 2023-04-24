use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::{commit_to_poly, compute_proof_single};

use arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
use arkworks::kzg_types::{ArkG1, ArkG2, FsFr};
use arkworks::utils::PolyData;

fn commit_to_poly_(c: &mut Criterion) {
    commit_to_poly::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
        c,
        &generate_trusted_setup,
    );
}

fn compute_proof_single_(c: &mut Criterion) {
    compute_proof_single::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(
        c,
        &generate_trusted_setup,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = commit_to_poly_, compute_proof_single_
}

criterion_main!(benches);
