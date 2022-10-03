use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::das::bench_das_extension;
use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::zkfr::blsScalar;

fn bench_das_extension_(c: &mut Criterion) {
    bench_das_extension::<blsScalar, ZkFFTSettings>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_das_extension_
}

criterion_main!(benches);
