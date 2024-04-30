use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use rust_kzg_arkworks3::kzg_proofs::FFTSettings;
use rust_kzg_arkworks3::kzg_types::{ArkFr, ArkG1};

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<ArkFr, FFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<ArkFr, ArkG1, FFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
