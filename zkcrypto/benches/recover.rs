use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::poly::ZPoly;
use zkcrypto::zkfr::blsScalar;

pub fn bench_recover_(c: &mut Criterion) {
    bench_recover::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>(c)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
