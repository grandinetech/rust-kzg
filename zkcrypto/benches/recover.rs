use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
use rust_kzg_zkcrypto::poly::ZPoly;
use rust_kzg_zkcrypto::zkfr::blsScalar;

pub fn bench_recover_(c: &mut Criterion) {
    bench_recover::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>(c)
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_recover_
}

criterion_main!(benches);
