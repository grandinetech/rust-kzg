use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::{bench_das_extension};
use blst_from_scratch::kzg_types::{FsFFTSettings, FsFr};

fn bench_das_extension_(c: &mut Criterion) {
    bench_das_extension::<FsFr, FsFFTSettings>(c)
}

criterion_group!(benches, bench_das_extension_);
criterion_main!(benches);