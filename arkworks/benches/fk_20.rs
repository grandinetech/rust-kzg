use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};
use rust_kzg_arkworks::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
use rust_kzg_arkworks::kzg_proofs::{generate_trusted_setup, FFTSettings, KZGSettings};
use rust_kzg_arkworks::kzg_types::{ArkFr, ArkG1, ArkG2};
use rust_kzg_arkworks::utils::PolyData;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<
        ArkFr,
        ArkG1,
        ArkG2,
        PolyData,
        FFTSettings,
        KZGSettings,
        KzgFK20SingleSettings,
    >(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<ArkFr, ArkG1, ArkG2, PolyData, FFTSettings, KZGSettings, KzgFK20MultiSettings>(
        c,
        &generate_trusted_setup,
    )
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
