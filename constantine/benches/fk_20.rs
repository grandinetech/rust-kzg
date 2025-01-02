use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};

use rust_kzg_constantine::eip_7594::CtBackend;
use rust_kzg_constantine::types::fk20_multi_settings::CtFK20MultiSettings;
use rust_kzg_constantine::types::fk20_single_settings::CtFK20SingleSettings;
use rust_kzg_constantine::utils::generate_trusted_setup;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<CtBackend, CtFK20SingleSettings>(c, &generate_trusted_setup)
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<CtBackend, CtFK20MultiSettings>(c, &generate_trusted_setup)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fk_single_da_, bench_fk_multi_da_
}

criterion_main!(benches);
