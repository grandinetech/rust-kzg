use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};

use rust_kzg_blst::types::fft_settings::FsFFTSettings;
use rust_kzg_blst::types::fk20_multi_settings::FsFK20MultiSettings;
use rust_kzg_blst::types::fk20_single_settings::FsFK20SingleSettings;
use rust_kzg_blst::types::fp::FsFp;
use rust_kzg_blst::types::fr::FsFr;
use rust_kzg_blst::types::g1::{FsG1, FsG1Affine};
use rust_kzg_blst::types::g2::FsG2;
use rust_kzg_blst::types::kzg_settings::FsKZGSettings;
use rust_kzg_blst::types::poly::FsPoly;
use rust_kzg_blst::utils::generate_trusted_setup;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<
        FsFr,
        FsG1,
        FsG2,
        FsPoly,
        FsFFTSettings,
        FsKZGSettings,
        FsFK20SingleSettings,
        FsFp,
        FsG1Affine,
    >(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<
        FsFr,
        FsG1,
        FsG2,
        FsPoly,
        FsFFTSettings,
        FsKZGSettings,
        FsFK20MultiSettings,
        FsFp,
        FsG1Affine,
    >(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
