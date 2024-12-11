use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};

use rust_kzg_mcl::eip_7594::MclBackend;
use rust_kzg_mcl::types::fk20_multi_settings::FsFK20MultiSettings;
use rust_kzg_mcl::types::fk20_single_settings::FsFK20SingleSettings;
use rust_kzg_mcl::utils::generate_trusted_setup;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<MclBackend, FsFK20SingleSettings>(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<MclBackend, FsFK20MultiSettings>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
