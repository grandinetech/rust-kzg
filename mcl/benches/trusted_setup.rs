use criterion::{criterion_group, criterion_main, Criterion};
use kzg::eip_4844::load_trusted_setup_rust;
use kzg_bench::benches::trusted_setup::bench_load_trusted_setup;
use rust_kzg_mcl::{
    eip_4844::load_trusted_setup_filename_rust,
    types::{
        fft_settings::FsFFTSettings,
        fp::FsFp,
        fr::FsFr,
        g1::{FsG1, FsG1Affine},
        g2::FsG2,
        kzg_settings::FsKZGSettings,
        poly::FsPoly,
    },
};

fn bench_load_trusted_setup_(c: &mut Criterion) {
    bench_load_trusted_setup::<
        FsFr,
        FsG1,
        FsG2,
        FsPoly,
        FsFFTSettings,
        FsKZGSettings,
        FsFp,
        FsG1Affine,
    >(
        c,
        &load_trusted_setup_filename_rust,
        &load_trusted_setup_rust,
    );
}

criterion_group!(benches, bench_load_trusted_setup_);
criterion_main!(benches);
