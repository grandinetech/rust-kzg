use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::recover::bench_recover;

use zkcrypto::fftsettings::ZkFFTSettings;
use zkcrypto::zkfr::blsScalar;
use zkcrypto::poly::ZPoly;


pub fn bench_recover_(c: &mut Criterion) {
    bench_recover::<blsScalar, ZkFFTSettings, ZPoly, ZPoly>(c)
}


criterion_group!(benches, bench_recover_);
criterion_main!(benches);