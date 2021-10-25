use criterion::{Criterion, criterion_group, criterion_main};
use kzg::Fr;
use kzg_bench::benches::fft::bench_fft_fr;
use kzg_from_scratch::kzg_types::{FsFFTSettings, FsFr};

fn bench_fft_fr_(c: &mut Criterion) {
    bench_fft_fr::<FsFr, FsFFTSettings>(c);
}

criterion_group!(benches, bench_fft_fr_);
criterion_main!(benches);