use criterion::{Criterion, criterion_group, criterion_main};
use kzg_bench::benches::kzg::kzg_proof;

use arkworks::kzg_types::{FsFr, ArkG1, ArkG2};
use arkworks::utils::PolyData;
use arkworks::kzg_proofs::{FFTSettings, KZGSettings, generate_trusted_setup};

fn kzg_proof_(c: &mut Criterion) {
    kzg_proof::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings>(c, &generate_trusted_setup);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(5);
    targets = kzg_proof_
}

criterion_main!(benches);
