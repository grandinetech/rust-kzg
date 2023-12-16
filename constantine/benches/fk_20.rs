use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fk20::{bench_fk_multi_da, bench_fk_single_da};

use rust_kzg_blst::types::fft_settings::CtFFTSettings;
use rust_kzg_blst::types::fk20_multi_settings::CtFK20MultiSettings;
use rust_kzg_blst::types::fk20_single_settings::CtFK20SingleSettings;
use rust_kzg_blst::types::fr::CtFr;
use rust_kzg_blst::types::g1::CtG1;
use rust_kzg_blst::types::g2::CtG2;
use rust_kzg_blst::types::kzg_settings::CtKZGSettings;
use rust_kzg_blst::types::poly::CtPoly;
use rust_kzg_blst::utils::generate_trusted_setup;

fn bench_fk_single_da_(c: &mut Criterion) {
    bench_fk_single_da::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFK20SingleSettings>(
        c,
        &generate_trusted_setup,
    )
}

fn bench_fk_multi_da_(c: &mut Criterion) {
    bench_fk_multi_da::<CtFr, CtG1, CtG2, CtPoly, CtFFTSettings, CtKZGSettings, CtFK20MultiSettings>(
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
