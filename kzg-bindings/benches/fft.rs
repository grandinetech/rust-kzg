use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::bench_fft_fr;
use kzg_bindings::finite::BlstFr;
use kzg_bindings::fftsettings::KzgFFTSettings;

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<BlstFr, KzgFFTSettings>(c);
}

criterion_group!(benches, bench_fft_fr_);
criterion_main!(benches);
