use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use rust_kzg_mcl::types::fft_settings::MclFFTSettings;
use rust_kzg_mcl::types::fr::MclFr;
use rust_kzg_mcl::types::g1::MclG1;

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<MclFr, MclFFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<MclFr, MclG1, MclFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
