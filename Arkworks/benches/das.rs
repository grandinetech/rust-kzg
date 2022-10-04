use arkworks::kzg_proofs::FFTSettings;
use arkworks::kzg_types::FsFr;
use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;
// use arkworks::das::

fn bench_das_extension_(c: &mut Criterion) {
    bench_das_extension::<FsFr, FFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_das_extension_
}

criterion_main!(benches);
