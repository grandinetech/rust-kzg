use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use rust_kzg_blst::types::fft_settings::CtFFTSettings;
use rust_kzg_blst::types::fr::CtFr;
use rust_kzg_blst::types::g1::CtG1;

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<CtFr, CtFFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<CtFr, CtG1, CtFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
