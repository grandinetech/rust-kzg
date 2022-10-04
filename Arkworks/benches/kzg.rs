use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::kzg::kzg_proof;

use arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
use arkworks::kzg_types::{ArkG1, ArkG2, FsFr};
use arkworks::utils::PolyData;

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(c, &generate_trusted_setup);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = kzg_proof_
}

criterion_main!(benches);
