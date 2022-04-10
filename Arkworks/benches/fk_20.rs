use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{fk_single_da, fk_multi_da};
use arkworks::kzg_types::{FsFr, ArkG1, ArkG2};
use arkworks::kzg_proofs::{FFTSettings, KZGSettings, generate_trusted_setup};
use arkworks::fk20_proofs::{KzgFK20SingleSettings, KzgFK20MultiSettings};
use arkworks::utils::PolyData;

fn fk_single_da_(c: &mut Criterion) {
    fk_single_da::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, KzgFK20SingleSettings>(c, &generate_trusted_setup)
}

fn fk_multi_da_(c: &mut Criterion) {
    fk_multi_da::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, KzgFK20MultiSettings>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(2);
    targets = fk_single_da_, fk_multi_da_
}

criterion_main!(benches);