use criterion::{criterion_group, criterion_main, Criterion};
use kzg::eip_4844::load_trusted_setup_rust;
use kzg_bench::benches::trusted_setup::bench_load_trusted_setup;
use rust_kzg_mcl::{
    eip_4844::load_trusted_setup_filename_rust,
    types::{
        fft_settings::MclFFTSettings,
        fp::MclFp,
        fr::MclFr,
        g1::{MclG1, MclG1Affine},
        g2::MclG2,
        kzg_settings::MclKZGSettings,
        poly::MclPoly,
    },
};

fn bench_load_trusted_setup_(c: &mut Criterion) {
    bench_load_trusted_setup::<
        MclFr,
        MclG1,
        MclG2,
        MclPoly,
        MclFFTSettings,
        MclKZGSettings,
        MclFp,
        MclG1Affine,
    >(
        c,
        &load_trusted_setup_filename_rust,
        &load_trusted_setup_rust,
    );
}

criterion_group!(benches, bench_load_trusted_setup_);
criterion_main!(benches);
