use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};

use blst_rust::types::fft_settings::FsFFTSettings;
use blst_rust::types::fk20_multi_settings::FsFK20MultiSettings;
use blst_rust::types::fk20_single_settings::FsFK20SingleSettings;
use blst_rust::types::fr::FsFr;
use blst_rust::types::g1::FsG1;
use blst_rust::types::g2::FsG2;
use blst_rust::types::kzg_settings::FsKZGSettings;
use blst_rust::types::poly::FsPoly;
use blst_rust::utils::generate_trusted_setup;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFK20SingleSettings>(
        c,
        &generate_trusted_setup,
    )
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<FsFr, FsG1, FsG2, FsPoly, FsFFTSettings, FsKZGSettings, FsFK20MultiSettings>(
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
