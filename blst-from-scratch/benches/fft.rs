use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::fft::{bench_fft_fr, bench_fft_g1};
use blst_from_scratch::kzg_types::{FsFFTSettings, FsFr, FsG1};

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<FsFr, FsFFTSettings>(c);
}

fn bench_fft_g1_(c: &mut Criterion) {
    bench_fft_g1::<FsFr, FsG1, FsFFTSettings>(c);
}

criterion_group!(benches, bench_fft_fr_, bench_fft_g1_);
criterion_main!(benches);
