use criterion::{criterion_group, criterion_main, Criterion};
use kzg::{FFTSettings};

fn fft_fr(c: &mut Criterion) {
    c.bench_function(
        "fft_fr",
        |b| b.iter(|| {
            FFTSettings::bench_fft_fr(8)
        })
    );
}

criterion_group!(benches, fft_fr);
criterion_main!(benches);
