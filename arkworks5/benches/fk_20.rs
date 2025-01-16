use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};
use rust_kzg_arkworks5::eip_7594::ArkBackend;
use rust_kzg_arkworks5::fk20_proofs::{KzgFK20MultiSettings, KzgFK20SingleSettings};
use rust_kzg_arkworks5::kzg_proofs::generate_trusted_setup;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<ArkBackend, KzgFK20SingleSettings>(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<ArkBackend, KzgFK20MultiSettings>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
