use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};
use rust_kzg_arkworks3::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
use rust_kzg_arkworks3::kzg_proofs::{generate_trusted_setup, LFFTSettings as FFTSettings, LKZGSettings as KZGSettings};
use rust_kzg_arkworks3::kzg_types::{ArkFp, ArkFr, ArkG1, ArkG1Affine, ArkG2};
use rust_kzg_arkworks3::utils::PolyData;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<
        ArkFr,
        ArkG1,
        ArkG2,
        PolyData,
        FFTSettings,
        KZGSettings,
        KzgFK20SingleSettings,
        ArkFp,
        ArkG1Affine,
    >(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<
        ArkFr,
        ArkG1,
        ArkG2,
        PolyData,
        FFTSettings,
        KZGSettings,
        KzgFK20MultiSettings,
        ArkFp,
        ArkG1Affine,
    >(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
