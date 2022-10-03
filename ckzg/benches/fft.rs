use ckzg::consts::BlstP1;
use ckzg::fftsettings::KzgFFTSettings;
use ckzg::finite::BlstFr;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<BlstFr, KzgFFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<BlstFr, BlstP1, KzgFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_fft_fr_, bench_fft_g1_
}

criterion_main!(benches);
