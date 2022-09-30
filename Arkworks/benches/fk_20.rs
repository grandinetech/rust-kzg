use arkworks::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
use arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
use arkworks::kzg_types::{ArkG1, ArkG2, FsFr};
use arkworks::utils::PolyData;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{fk_multi_da, fk_single_da};

fn fk_single_da_(c: &mut Criterion) {
    fk_single_da::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, KzgFK20SingleSettings>(
        c,
        &generate_trusted_setup,
    )
}

fn fk_multi_da_(c: &mut Criterion) {
    fk_multi_da::<FsFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, KzgFK20MultiSettings>(
        c,
        &generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = fk_single_da_, fk_multi_da_
}

criterion_main!(benches);
