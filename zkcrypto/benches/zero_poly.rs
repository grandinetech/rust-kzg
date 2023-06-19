use criterion::{criterion_group, criterion_main, Criterion};
use kzg_bench::benches::zero_poly::bench_zero_poly;
use rust_kzg_zkcrypto::fftsettings::ZkFFTSettings;
use rust_kzg_zkcrypto::poly::ZPoly;
use rust_kzg_zkcrypto::zkfr::blsScalar;

fn bench_zero_poly_(c: &mut Criterion) {
    bench_zero_poly::<blsScalar, ZkFFTSettings, ZPoly>(c);
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10);
    targets = bench_zero_poly_
}

criterion_main!(benches);
