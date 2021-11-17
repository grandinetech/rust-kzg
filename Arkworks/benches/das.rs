use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;
use arkworks::kzg_proofs::FFTSettings;
use arkworks::kzg_types::FsFr;
// use arkworks::das::

fn bench_das_extension_(c: &mut Criterion) {
    bench_das_extension::<FsFr, FFTSettings>(c);
}

criterion_group!(benches, bench_das_extension_);
criterion_main!(benches);
