use blst_from_scratch::types::{fft_settings::FsFFTSettings, fr::FsFr, poly::FsPoly};
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

pub fn bench_recover_(c: &mut Criterion) {
    bench_recover::<FsFr, FsFFTSettings, FsPoly, FsPoly>(c)
}


criterion_group!(benches, bench_recover_);
criterion_main!(benches);
